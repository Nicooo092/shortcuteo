use windows::core::{HSTRING, PCWSTR};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

pub fn open(target: &str, args: &str) -> Result<(), String> {
    let file = HSTRING::from(target);
    let params = HSTRING::from(args);
    let verb = HSTRING::from("open");
    // blocking, but returns as soon as the target is handed off, not when it exits
    let code = unsafe {
        ShellExecuteW(
            None,
            &verb,
            &file,
            if args.is_empty() {
                PCWSTR::null()
            } else {
                PCWSTR(params.as_ptr())
            },
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    // values above 32 mean success, the rest are legacy error codes
    match code.0 as usize {
        n if n > 32 => Ok(()),
        2 | 3 => Err(format!("'{target}' was not found")),
        5 => Err(format!("access to '{target}' was denied")),
        31 => Err(format!("no app is associated with '{target}'")),
        n => Err(format!("could not open '{target}' (code {n})")),
    }
}
