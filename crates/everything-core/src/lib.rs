pub mod crawler;
pub mod entry;
pub mod index;
pub mod search;
pub mod watcher;

pub use crawler::{Crawler, CrawlerConfig, CrawlStats, DEFAULT_EXCLUDED_ROOTS};
pub use entry::FileEntry;
pub use index::Database;
pub use search::{FileCategory, SearchEngine, SearchQuery, SearchResult};
pub use watcher::FileWatcher;
