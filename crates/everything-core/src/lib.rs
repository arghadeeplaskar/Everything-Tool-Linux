pub mod crawler;
pub mod entry;
pub mod index;
pub mod mounts;
pub mod search;
pub mod watcher;

pub use crawler::{Crawler, CrawlerConfig, CrawlStats, DEFAULT_EXCLUDED_ROOTS};
pub use entry::{CompactEntry, DirectoryStore, FileEntry, FLAG_IS_DIR, FLAG_IS_REMOVABLE, FLAG_IS_SYMLINK};
pub use index::{Database, IndexData};
pub use mounts::{MountEntry, MountTable, REAL_FS_TYPES, VIRTUAL_FS_TYPES};
pub use search::{FileCategory, SearchEngine, SearchQuery, SearchResult};
pub use watcher::{FileWatcher, WatcherHealth};
