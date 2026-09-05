use crate::entry::FileEntry;
use glob::Pattern;
use rayon::prelude::*;
use regex::Regex;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileCategory {
    #[default]
    All,
    Folders,
    Documents,
    Images,
    Audio,
    Video,
    Archives,
    Code,
}

impl FileCategory {
    pub fn matches(&self, entry: &FileEntry) -> bool {
        match self {
            FileCategory::All => true,
            FileCategory::Folders => entry.is_dir,
            FileCategory::Documents => {
                if entry.is_dir {
                    return false;
                }
                let ext = entry.extension().unwrap_or("").to_ascii_lowercase();
                matches!(
                    ext.as_str(),
                    "pdf" | "doc" | "docx" | "txt" | "md" | "odt" | "rtf" | "epub" | "csv" | "xlsx" | "pptx" | "ods" | "log"
                )
            }
            FileCategory::Images => {
                if entry.is_dir {
                    return false;
                }
                let ext = entry.extension().unwrap_or("").to_ascii_lowercase();
                matches!(
                    ext.as_str(),
                    "png" | "jpg" | "jpeg" | "svg" | "gif" | "webp" | "bmp" | "ico" | "tiff" | "raw" | "heic"
                )
            }
            FileCategory::Audio => {
                if entry.is_dir {
                    return false;
                }
                let ext = entry.extension().unwrap_or("").to_ascii_lowercase();
                matches!(
                    ext.as_str(),
                    "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "wma" | "opus"
                )
            }
            FileCategory::Video => {
                if entry.is_dir {
                    return false;
                }
                let ext = entry.extension().unwrap_or("").to_ascii_lowercase();
                matches!(
                    ext.as_str(),
                    "mp4" | "mkv" | "webm" | "avi" | "mov" | "flv" | "wmv" | "m4v"
                )
            }
            FileCategory::Archives => {
                if entry.is_dir {
                    return false;
                }
                let ext = entry.extension().unwrap_or("").to_ascii_lowercase();
                matches!(
                    ext.as_str(),
                    "zip" | "tar" | "gz" | "7z" | "rar" | "xz" | "bz2" | "iso" | "deb" | "rpm" | "zst"
                )
            }
            FileCategory::Code => {
                if entry.is_dir {
                    return false;
                }
                let ext = entry.extension().unwrap_or("").to_ascii_lowercase();
                matches!(
                    ext.as_str(),
                    "rs" | "py" | "c" | "cpp" | "h" | "hpp" | "js" | "ts" | "jsx" | "tsx"
                        | "html" | "css" | "json" | "toml" | "yaml" | "yml" | "sh" | "bash"
                        | "go" | "java" | "kt" | "swift" | "sql" | "xml" | "php"
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub raw: String,
    pub case_sensitive: bool,
    pub is_regex: bool,
    pub is_wildcard: bool,
    pub dir_only: bool,
    pub file_only: bool,
    pub category: FileCategory,
    pub max_results: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            raw: String::new(),
            case_sensitive: false,
            is_regex: false,
            is_wildcard: false,
            dir_only: false,
            file_only: false,
            category: FileCategory::All,
            max_results: 10_000,
        }
    }
}

pub struct SearchResult {
    pub entries: Vec<FileEntry>,
    pub elapsed_micros: u128,
    pub total_scanned: usize,
}

pub struct SearchEngine;

impl SearchEngine {
    pub fn execute(entries: &[FileEntry], query: &SearchQuery) -> SearchResult {
        let start = Instant::now();

        if query.raw.is_empty() {
            if query.category != FileCategory::All || query.dir_only || query.file_only {
                let matched: Vec<FileEntry> = entries
                    .par_iter()
                    .filter(|e| {
                        if query.dir_only && !e.is_dir {
                            return false;
                        }
                        if query.file_only && e.is_dir {
                            return false;
                        }
                        query.category.matches(e)
                    })
                    .take_any(query.max_results)
                    .cloned()
                    .collect();

                return SearchResult {
                    entries: matched,
                    elapsed_micros: start.elapsed().as_micros(),
                    total_scanned: entries.len(),
                };
            }

            let slice = if entries.len() > query.max_results {
                &entries[..query.max_results]
            } else {
                entries
            };
            return SearchResult {
                entries: slice.to_vec(),
                elapsed_micros: start.elapsed().as_micros(),
                total_scanned: entries.len(),
            };
        }

        let has_slash = query.raw.contains('/');

        // 1. Regex Mode
        if query.is_regex {
            let regex_result = if query.case_sensitive {
                Regex::new(&query.raw)
            } else {
                Regex::new(&format!("(?i){}", &query.raw))
            };

            if let Ok(re) = regex_result {
                let matched: Vec<FileEntry> = entries
                    .par_iter()
                    .filter(|e| {
                        if query.dir_only && !e.is_dir {
                            return false;
                        }
                        if query.file_only && e.is_dir {
                            return false;
                        }
                        if !query.category.matches(e) {
                            return false;
                        }
                        if has_slash {
                            re.is_match(&e.full_path().to_string_lossy())
                        } else {
                            re.is_match(&e.name)
                        }
                    })
                    .take_any(query.max_results)
                    .cloned()
                    .collect();

                return SearchResult {
                    entries: matched,
                    elapsed_micros: start.elapsed().as_micros(),
                    total_scanned: entries.len(),
                };
            }
        }

        // 2. Wildcard Mode (if contains * or ?)
        let contains_wildcards = query.raw.contains('*') || query.raw.contains('?');
        if query.is_wildcard || contains_wildcards {
            let pattern_str = if query.case_sensitive {
                query.raw.clone()
            } else {
                query.raw.to_lowercase()
            };

            if let Ok(glob_pattern) = Pattern::new(&pattern_str) {
                let matched: Vec<FileEntry> = entries
                    .par_iter()
                    .filter(|e| {
                        if query.dir_only && !e.is_dir {
                            return false;
                        }
                        if query.file_only && e.is_dir {
                            return false;
                        }
                        if !query.category.matches(e) {
                            return false;
                        }

                        if has_slash {
                            let target = if query.case_sensitive {
                                e.full_path().to_string_lossy().to_string()
                            } else {
                                e.full_path().to_string_lossy().to_lowercase()
                            };
                            glob_pattern.matches(&target)
                        } else {
                            let target = if query.case_sensitive {
                                &e.name
                            } else {
                                &e.name_lower
                            };
                            glob_pattern.matches(target)
                        }
                    })
                    .take_any(query.max_results)
                    .cloned()
                    .collect();

                return SearchResult {
                    entries: matched,
                    elapsed_micros: start.elapsed().as_micros(),
                    total_scanned: entries.len(),
                };
            }
        }

        // 3. Ultra-Fast Substring Matching (Default Mode)
        let query_term = if query.case_sensitive {
            query.raw.clone()
        } else {
            query.raw.to_lowercase()
        };

        // Split multiple search keywords separated by space (like Everything: `test doc pdf`)
        let terms: Vec<&str> = query_term.split_whitespace().collect();

        let matched: Vec<FileEntry> = entries
            .par_iter()
            .filter(|e| {
                if query.dir_only && !e.is_dir {
                    return false;
                }
                if query.file_only && e.is_dir {
                    return false;
                }
                if !query.category.matches(e) {
                    return false;
                }

                if terms.is_empty() {
                    return true;
                }

                if has_slash {
                    let full_str = if query.case_sensitive {
                        e.full_path().to_string_lossy().to_string()
                    } else {
                        e.full_path().to_string_lossy().to_lowercase()
                    };
                    terms.iter().all(|&t| full_str.contains(t))
                } else if query.case_sensitive {
                    terms.iter().all(|&t| e.name.contains(t))
                } else {
                    terms.iter().all(|&t| e.name_lower.contains(t))
                }
            })
            .take_any(query.max_results)
            .cloned()
            .collect();

        SearchResult {
            entries: matched,
            elapsed_micros: start.elapsed().as_micros(),
            total_scanned: entries.len(),
        }
    }
}
