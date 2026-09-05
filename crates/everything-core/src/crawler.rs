use crate::entry::FileEntry;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Directories that should never be indexed on Linux
pub const DEFAULT_EXCLUDED_ROOTS: &[&str] = &[
    "/proc",
    "/sys",
    "/dev",
    "/run",
    "/tmp",
    "/var/run",
    "/var/lock",
    "/var/lib/docker",
    "/var/lib/containerd",
    "/.snapshots",
];

pub struct CrawlerConfig {
    pub excluded_paths: HashSet<PathBuf>,
    pub follow_symlinks: bool,
    pub max_depth: Option<usize>,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        let mut excluded_paths = HashSet::new();
        for &path in DEFAULT_EXCLUDED_ROOTS {
            excluded_paths.insert(PathBuf::from(path));
        }
        Self {
            excluded_paths,
            follow_symlinks: false,
            max_depth: None,
        }
    }
}

pub struct CrawlStats {
    pub total_files: usize,
    pub total_dirs: usize,
    pub elapsed_ms: u128,
}

pub struct Crawler {
    config: CrawlerConfig,
}

impl Crawler {
    pub fn new(config: CrawlerConfig) -> Self {
        Self { config }
    }

    /// Crawls a list of target paths in parallel
    pub fn crawl_roots(&self, roots: &[PathBuf]) -> (Vec<FileEntry>, CrawlStats) {
        let start = Instant::now();
        let total_files = Arc::new(AtomicUsize::new(0));
        let total_dirs = Arc::new(AtomicUsize::new(0));

        let entries: Vec<FileEntry> = roots
            .par_iter()
            .flat_map(|root| {
                if !root.exists() || self.is_excluded(root) {
                    return Vec::new();
                }
                self.crawl_dir_recursive(root, 0, &total_files, &total_dirs)
            })
            .collect();

        let stats = CrawlStats {
            total_files: total_files.load(Ordering::Relaxed),
            total_dirs: total_dirs.load(Ordering::Relaxed),
            elapsed_ms: start.elapsed().as_millis(),
        };

        (entries, stats)
    }

    fn is_excluded(&self, path: &Path) -> bool {
        if self.config.excluded_paths.contains(path) {
            return true;
        }
        false
    }

    fn crawl_dir_recursive(
        &self,
        dir: &Path,
        current_depth: usize,
        file_counter: &Arc<AtomicUsize>,
        dir_counter: &Arc<AtomicUsize>,
    ) -> Vec<FileEntry> {
        // Safety ceiling against extremely deep directory trees or recursion
        if current_depth > 100 {
            return Vec::new();
        }

        if let Some(max_depth) = self.config.max_depth {
            if current_depth > max_depth {
                return Vec::new();
            }
        }

        let read_dir = match fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => return Vec::new(), // Permission denied or I/O error
        };

        let mut local_entries = Vec::with_capacity(64);
        let mut subdirs = Vec::new();

        for entry_res in read_dir {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            if self.is_excluded(&path) {
                continue;
            }

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            let is_symlink = file_type.is_symlink();
            if is_symlink && !self.config.follow_symlinks {
                continue;
            }

            let is_dir = file_type.is_dir();
            let metadata = entry.metadata().ok();

            if let Some(fe) = FileEntry::new(&path, metadata.as_ref()) {
                if is_dir {
                    dir_counter.fetch_add(1, Ordering::Relaxed);
                    subdirs.push(path);
                } else {
                    file_counter.fetch_add(1, Ordering::Relaxed);
                }
                local_entries.push(fe);
            }
        }

        // Parallelize subdirectories across all CPU threads
        if current_depth < 6 && subdirs.len() > 1 {
            let subdir_entries: Vec<FileEntry> = subdirs
                .into_par_iter()
                .flat_map(|sub| self.crawl_dir_recursive(&sub, current_depth + 1, file_counter, dir_counter))
                .collect();
            local_entries.extend(subdir_entries);
        } else {
            for sub in subdirs {
                let sub_results = self.crawl_dir_recursive(&sub, current_depth + 1, file_counter, dir_counter);
                local_entries.extend(sub_results);
            }
        }

        local_entries
    }
}
