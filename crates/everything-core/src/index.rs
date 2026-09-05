use crate::crawler::{Crawler, CrawlerConfig, CrawlStats};
use crate::entry::FileEntry;
use crate::search::{SearchEngine, SearchQuery, SearchResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Database {
    entries: Arc<RwLock<Vec<FileEntry>>>,
    config: Arc<CrawlerConfig>,
}

impl Default for Database {
    fn default() -> Self {
        Self::new(CrawlerConfig::default())
    }
}

impl Database {
    pub fn new(config: CrawlerConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            config: Arc::new(config),
        }
    }

    /// Indexes the specified root directories
    pub fn index_roots(&self, roots: &[PathBuf]) -> CrawlStats {
        let crawler = Crawler::new(CrawlerConfig {
            excluded_paths: self.config.excluded_paths.clone(),
            follow_symlinks: self.config.follow_symlinks,
            max_depth: self.config.max_depth,
        });

        let (crawled, stats) = crawler.crawl_roots(roots);
        {
            let mut write_guard = self.entries.write().unwrap();
            *write_guard = crawled;
        }
        stats
    }

    /// Performs search against in-memory index
    pub fn search(&self, query: &SearchQuery) -> SearchResult {
        let read_guard = self.entries.read().unwrap();
        SearchEngine::execute(&read_guard, query)
    }

    /// Total entries currently indexed
    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Inserts or updates an entry for a given path
    pub fn upsert_path(&self, path: &Path) {
        if let Some(entry) = FileEntry::new(path, None) {
            let mut write_guard = self.entries.write().unwrap();
            let full_path = entry.full_path();
            if let Some(pos) = write_guard.iter().position(|e| e.full_path() == full_path) {
                write_guard[pos] = entry;
            } else {
                write_guard.push(entry);
            }
        }
    }

    /// Batch inserts or updates entries with a single write lock acquisition
    pub fn batch_upsert(&self, paths: &[PathBuf]) {
        let new_entries: Vec<FileEntry> = paths
            .iter()
            .filter_map(|p| FileEntry::new(p, None))
            .collect();

        if new_entries.is_empty() {
            return;
        }

        let mut write_guard = self.entries.write().unwrap();
        for entry in new_entries {
            let full_path = entry.full_path();
            if let Some(pos) = write_guard.iter().position(|e| e.full_path() == full_path) {
                write_guard[pos] = entry;
            } else {
                write_guard.push(entry);
            }
        }
    }

    /// Removes an entry matching a given path
    pub fn remove_path(&self, path: &Path) {
        let mut write_guard = self.entries.write().unwrap();
        write_guard.retain(|e| e.full_path() != path);
    }

    /// Batch removes entries with a single write lock acquisition
    pub fn batch_remove(&self, paths: &[PathBuf]) {
        if paths.is_empty() {
            return;
        }
        let remove_set: std::collections::HashSet<&PathBuf> = paths.iter().collect();
        let mut write_guard = self.entries.write().unwrap();
        write_guard.retain(|e| !remove_set.contains(&e.full_path()));
    }

    /// Save index to cache file for fast restarts
    pub fn save_cache(&self, cache_file: &Path) -> std::io::Result<()> {
        if let Some(parent) = cache_file.parent() {
            fs::create_dir_all(parent)?;
        }
        let read_guard = self.entries.read().unwrap();
        let json = serde_json::to_string(&*read_guard)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(cache_file, json)
    }

    /// Load index from cache file
    pub fn load_cache(&self, cache_file: &Path) -> std::io::Result<usize> {
        let content = fs::read_to_string(cache_file)?;
        let loaded: Vec<FileEntry> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let count = loaded.len();
        let mut write_guard = self.entries.write().unwrap();
        *write_guard = loaded;
        Ok(count)
    }
}
