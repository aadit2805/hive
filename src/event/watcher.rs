use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use tokio::sync::mpsc as tokio_mpsc;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};

use super::types::HiveEvent;

/// Watches a file for new JSON events and sends them to a channel
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    file_path: std::path::PathBuf,
    last_position: u64,
}

impl FileWatcher {
    /// Create a new file watcher that monitors the given path
    pub fn new(
        path: impl AsRef<Path>,
        event_tx: tokio_mpsc::Sender<HiveEvent>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = path.as_ref().to_path_buf();

        // Create the file if it doesn't exist
        if !file_path.exists() {
            std::fs::write(&file_path, "")?;
        }

        // Get initial file size
        let initial_position = std::fs::metadata(&file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let (tx, rx) = mpsc::channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default(),
        )?;

        let mut file_watcher = Self {
            _watcher: watcher,
            file_path: file_path.clone(),
            last_position: initial_position,
        };

        // Start watching the file
        file_watcher._watcher.watch(&file_path, RecursiveMode::NonRecursive)?;

        // Spawn a task to handle file change events
        let watch_path = file_path.clone();
        let mut last_pos = initial_position;

        tokio::spawn(async move {
            loop {
                // Check for notify events
                match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(_event) => {
                        // File changed, read new lines
                        if let Ok(new_events) = read_new_lines(&watch_path, &mut last_pos) {
                            for event in new_events {
                                if event_tx.send(event).await.is_err() {
                                    return; // Channel closed
                                }
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Periodically check for changes even without notify events
                        if let Ok(new_events) = read_new_lines(&watch_path, &mut last_pos) {
                            for event in new_events {
                                if event_tx.send(event).await.is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        return;
                    }
                }
            }
        });

        Ok(file_watcher)
    }

    /// Read all existing events from the file (for replay/initial load)
    pub fn read_all_events(&self) -> Vec<HiveEvent> {
        let mut events = Vec::new();

        if let Ok(file) = File::open(&self.file_path) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if line.trim().is_empty() {
                        continue;
                    }
                    if let Ok(event) = serde_json::from_str::<HiveEvent>(&line) {
                        events.push(event);
                    }
                }
            }
        }

        events
    }
}

/// Read new lines from the file starting at the given position
fn read_new_lines(
    path: &Path,
    last_position: &mut u64,
) -> Result<Vec<HiveEvent>, std::io::Error> {
    let mut events = Vec::new();

    let mut file = File::open(path)?;
    let current_size = file.metadata()?.len();

    // If file was truncated, start from beginning
    if current_size < *last_position {
        *last_position = 0;
    }

    // Seek to last known position
    file.seek(SeekFrom::Start(*last_position))?;

    let reader = BufReader::new(file);
    let mut bytes_read = *last_position;

    for line in reader.lines() {
        if let Ok(line) = line {
            bytes_read += line.len() as u64 + 1; // +1 for newline

            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<HiveEvent>(&line) {
                Ok(event) => events.push(event),
                Err(e) => {
                    eprintln!("Failed to parse event: {} - Line: {}", e, line);
                }
            }
        }
    }

    *last_position = bytes_read;

    Ok(events)
}
