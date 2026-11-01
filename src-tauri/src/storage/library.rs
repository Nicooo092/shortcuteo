use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::workflow::{self, Workflow};

pub const EXTENSION: &str = "shortcuteo";

/// Saved shortcuts, one `.shortcuteo` file per workflow, named after its id.
pub struct Library {
    dir: PathBuf,
}

impl Library {
    pub fn open(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Library { dir })
    }

    /// Every readable shortcut, sorted by name. Broken files are skipped, not fatal: one bad
    /// import should not hide the whole library.
    pub fn list(&self) -> Vec<Workflow> {
        let mut all: Vec<Workflow> = fs::read_dir(&self.dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == EXTENSION))
            .filter_map(|p| read(&p).ok())
            .collect();
        all.sort_by_key(|w| w.name.to_lowercase());
        all
    }

    pub fn find_by_name(&self, name: &str) -> Option<Workflow> {
        self.list()
            .into_iter()
            .find(|w| w.name.eq_ignore_ascii_case(name.trim()))
    }

    pub fn save(&self, wf: &Workflow) -> io::Result<()> {
        let path = self.path_for(&wf.id)?;
        // write then rename so a crash never leaves a half written shortcut
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, workflow::to_string(wf))?;
        fs::rename(tmp, path)
    }

    pub fn delete(&self, id: &str) -> io::Result<()> {
        match fs::remove_file(self.path_for(id)?) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }

    fn path_for(&self, id: &str) -> io::Result<PathBuf> {
        let safe = !id.is_empty()
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if !safe {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("bad shortcut id '{id}'"),
            ));
        }
        Ok(self.dir.join(format!("{id}.{EXTENSION}")))
    }
}

fn read(path: &Path) -> Result<Workflow, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    workflow::parse(&text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Tile;

    fn temp_library(tag: &str) -> Library {
        let dir =
            std::env::temp_dir().join(format!("shortcuteo-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        Library::open(dir).unwrap()
    }

    fn wf(id: &str, name: &str) -> Workflow {
        Workflow {
            schema_version: workflow::SCHEMA_VERSION,
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            icon: "sparkles".to_string(),
            color: Tile::Iris,
            actions: Vec::new(),
        }
    }

    #[test]
    fn save_list_find_delete() {
        let lib = temp_library("crud");
        lib.save(&wf("b", "Beta")).unwrap();
        lib.save(&wf("a", "alpha")).unwrap();
        fs::write(lib.dir.join("broken.shortcuteo"), "{nope").unwrap();

        let names: Vec<_> = lib.list().into_iter().map(|w| w.name).collect();
        assert_eq!(names, ["alpha", "Beta"]);
        assert_eq!(lib.find_by_name(" BETA ").unwrap().id, "b");

        lib.delete("b").unwrap();
        lib.delete("b").unwrap();
        assert!(lib.find_by_name("Beta").is_none());
        let _ = fs::remove_dir_all(&lib.dir);
    }

    #[test]
    fn rejects_path_traversal_ids() {
        let lib = temp_library("ids");
        assert!(lib.save(&wf("../evil", "x")).is_err());
        assert!(lib.delete("a/b").is_err());
        let _ = fs::remove_dir_all(&lib.dir);
    }
}
