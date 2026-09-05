use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub name_lower: String,
    pub parent: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub modified: u64, // Unix timestamp in seconds
}

impl FileEntry {
    pub fn new(path: &Path, metadata: Option<&std::fs::Metadata>) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().to_string();
        let name_lower = name.to_lowercase();
        let parent = path.parent().unwrap_or(Path::new("/")).to_path_buf();

        let (size, is_dir, modified) = if let Some(meta) = metadata {
            let size = meta.len();
            let is_dir = meta.is_dir();
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            (size, is_dir, modified)
        } else if let Ok(meta) = std::fs::symlink_metadata(path) {
            let size = meta.len();
            let is_dir = meta.is_dir();
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            (size, is_dir, modified)
        } else {
            (0, false, 0)
        };

        Some(Self {
            name,
            name_lower,
            parent,
            size,
            is_dir,
            modified,
        })
    }

    #[inline]
    pub fn full_path(&self) -> PathBuf {
        self.parent.join(&self.name)
    }

    #[inline]
    pub fn extension(&self) -> Option<&str> {
        let name = &self.name;
        let dot_pos = name.rfind('.')?;
        if dot_pos == 0 || dot_pos == name.len() - 1 {
            None
        } else {
            Some(&name[dot_pos + 1..])
        }
    }
}
