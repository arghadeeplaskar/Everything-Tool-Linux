use crate::index::Database;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;

pub struct FileWatcher {
    _watcher_thread: Option<thread::JoinHandle<()>>,
}

impl FileWatcher {
    /// Starts a background watcher thread that updates the database on filesystem changes
    pub fn start(db: Database, watch_roots: Vec<PathBuf>) -> notify::Result<Self> {
        let (tx, rx) = channel::<notify::Result<Event>>();

        let mut watcher = notify::recommended_watcher(tx)?;

        for root in &watch_roots {
            if root.exists() {
                let _ = watcher.watch(root, RecursiveMode::Recursive);
            }
        }

        let handle = thread::spawn(move || {
            let _watcher = watcher;
            let mut upsert_buffer = Vec::new();
            let mut remove_buffer = Vec::new();

            loop {
                match rx.recv_timeout(std::time::Duration::from_millis(150)) {
                    Ok(Ok(event)) => {
                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                upsert_buffer.extend(event.paths);
                            }
                            EventKind::Remove(_) => {
                                remove_buffer.extend(event.paths);
                            }
                            _ => {}
                        }

                        // Flush if buffer grows large
                        if upsert_buffer.len() >= 200 {
                            db.batch_upsert(&upsert_buffer);
                            upsert_buffer.clear();
                        }
                        if remove_buffer.len() >= 200 {
                            db.batch_remove(&remove_buffer);
                            remove_buffer.clear();
                        }
                    }
                    Ok(Err(_)) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Flush buffered events on timeout (quiescent period)
                        if !upsert_buffer.is_empty() {
                            db.batch_upsert(&upsert_buffer);
                            upsert_buffer.clear();
                        }
                        if !remove_buffer.is_empty() {
                            db.batch_remove(&remove_buffer);
                            remove_buffer.clear();
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }
        });

        Ok(Self {
            _watcher_thread: Some(handle),
        })
    }
}
