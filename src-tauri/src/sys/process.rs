use std::os::windows::process::CommandExt;
use std::process::Stdio;

use base64::Engine as _;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub code: Option<i32>,
}

impl Output {
    pub fn into_result(self) -> Result<String, String> {
        match self.code {
            Some(0) => Ok(self.stdout.trim_end().to_string()),
            code => {
                let detail = if self.stderr.trim().is_empty() {
                    &self.stdout
                } else {
                    &self.stderr
                };
                let code = code.map_or("killed".to_string(), |c| format!("exit code {c}"));
                Err(format!("{code}: {}", detail.trim()))
            }
        }
    }
}

/// Runs a PowerShell script. `stdin` is readable from the script as `$ShortcutInput`.
pub async fn powershell(script: &str, stdin: &str) -> Result<Output, String> {
    // -EncodedCommand sidesteps every quoting problem and keeps stdin free for the input
    let full = format!(
        "[Console]::OutputEncoding = New-Object Text.UTF8Encoding $false\n\
         $ShortcutInput = [Console]::In.ReadToEnd()\n\
         {script}"
    );
    let utf16: Vec<u8> = full.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let encoded = base64::engine::general_purpose::STANDARD.encode(utf16);

    let mut cmd = Command::new("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
    ])
    .args(["-EncodedCommand", &encoded]);
    run(cmd, stdin).await
}

/// Runs a command line through cmd.exe, so pipes and built-ins like `dir` work.
pub async fn cmd(command_line: &str, stdin: &str) -> Result<Output, String> {
    let mut cmd = Command::new("cmd.exe");
    // chcp keeps output in utf-8 instead of the oem code page
    cmd.as_std_mut()
        .raw_arg(format!("/D /S /C \"chcp 65001 >nul & {command_line}\""));
    run(cmd, stdin).await
}

async fn run(mut cmd: Command, stdin: &str) -> Result<Output, String> {
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .kill_on_drop(true);
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("could not start process: {e}"))?;
    if let Some(mut pipe) = child.stdin.take() {
        // a script that never reads stdin closes it early, that is fine
        let _ = pipe.write_all(stdin.as_bytes()).await;
    }
    let out = child.wait_with_output().await.map_err(|e| e.to_string())?;
    Ok(Output {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn powershell_reads_input_and_keeps_unicode() {
        let out = powershell("\"$ShortcutInput-été\".ToUpper()", "abc")
            .await
            .unwrap();
        assert_eq!(out.into_result().unwrap(), "ABC-ÉTÉ");
    }

    #[tokio::test]
    async fn powershell_errors_carry_stderr() {
        let out = powershell("Write-Error 'boom'; exit 3", "").await.unwrap();
        let err = out.into_result().unwrap_err();
        assert!(err.starts_with("exit code 3"), "{err}");
        assert!(err.contains("boom"), "{err}");
    }

    #[tokio::test]
    async fn cmd_supports_builtins_and_pipes() {
        let out = cmd("echo héllo | findstr h", "").await.unwrap();
        assert_eq!(out.into_result().unwrap(), "héllo");
    }
}
