//! # Nova Download Manager
//!
//! A comprehensive download manager with pause/resume support, progress
//! tracking, queue management, and file organization.
//!
//! ## Features
//!
//! - **Pause/Resume**: Support for HTTP Range requests to resume interrupted downloads
//! - **Progress tracking**: Real-time progress with speed and ETA calculation
//! - **Queue management**: Concurrent download limits and priority ordering
//! - **File organization**: Automatic naming, conflict resolution, directory management
//! - **History**: Persistent download history with status tracking
//!
//! ## Architecture
//!
//! The [`DownloadManager`] coordinates individual [`Download`] tasks. Each
//! download has a [`DownloadState`] and emits [`DownloadProgress`] updates.
//! The manager enforces concurrency limits and handles queue prioritization.

use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur during download operations.
#[derive(Error, Debug)]
pub enum DownloadError {
    /// A download was not found.
    #[error("download not found: {0}")]
    NotFound(String),

    /// The download has already been completed.
    #[error("download already completed: {0}")]
    AlreadyCompleted(String),

    /// The download cannot be resumed (no partial data or server does not support it).
    #[error("download cannot be resumed: {0}")]
    CannotResume(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Network error.
    #[error("network error: {0}")]
    NetworkError(String),

    /// JSON serialization error.
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Invalid download URL.
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    /// Insufficient disk space.
    #[error("insufficient disk space: required {required}, available {available}")]
    InsufficientSpace {
        /// Bytes required.
        required: u64,
        /// Bytes available.
        available: u64,
    },

    /// A generic error.
    #[error("download error: {0}")]
    Other(String),
}

/// Convenience type alias.
pub type Result<T> = std::result::Result<T, DownloadError>;

// ---------------------------------------------------------------------------
// Data models
// ---------------------------------------------------------------------------

/// The current state of a download.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadState {
    /// Download is queued and waiting to start.
    Queued,
    /// Download is actively transferring data.
    Downloading,
    /// Download has been paused by the user.
    Paused,
    /// Download completed successfully.
    Completed,
    /// Download failed with an error.
    Failed,
    /// Download was cancelled by the user.
    Cancelled,
}

/// Progress information for an active download.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// Total size in bytes, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,

    /// Bytes downloaded so far.
    pub downloaded_bytes: u64,

    /// Download speed in bytes per second.
    pub speed_bytes_per_sec: f64,

    /// Estimated time remaining in seconds, if total size is known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eta_secs: Option<f64>,

    /// Progress as a percentage (0.0 to 100.0), if total size is known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent: Option<f64>,

    /// When the download started.
    pub started_at: DateTime<Utc>,

    /// When progress was last updated.
    pub updated_at: DateTime<Utc>,
}

/// A single download task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    /// Unique identifier.
    pub id: String,

    /// Source URL of the download.
    pub url: String,

    /// Destination file path on disk.
    pub destination: PathBuf,

    /// Suggested filename from the server or user.
    pub filename: String,

    /// MIME type, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Current state of the download.
    pub state: DownloadState,

    /// Progress information.
    pub progress: DownloadProgress,

    /// Total size of the file in bytes, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,

    /// Where the download originated (e.g., the page URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,

    /// Priority in the queue (higher = more important).
    #[serde(default = "default_priority")]
    pub priority: u32,

    /// Whether this download supports resuming.
    #[serde(default)]
    pub supports_resume: bool,

    /// Error message if the download failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// When the download was created.
    pub created_at: DateTime<Utc>,

    /// When the download completed, if it has.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

fn default_priority() -> u32 {
    0
}

impl Download {
    /// Returns true if the download is in a terminal state.
    pub fn is_finished(&self) -> bool {
        matches!(
            self.state,
            DownloadState::Completed | DownloadState::Failed | DownloadState::Cancelled
        )
    }

    /// Returns true if the download can be paused.
    pub fn can_pause(&self) -> bool {
        self.state == DownloadState::Downloading
    }

    /// Returns true if the download can be resumed.
    pub fn can_resume(&self) -> bool {
        self.state == DownloadState::Paused && self.supports_resume
    }

    /// Returns the progress as a percentage string.
    pub fn progress_percent(&self) -> String {
        match self.progress.percent {
            Some(p) => format!("{:.1}%", p),
            None => "Unknown".to_string(),
        }
    }

    /// Returns a human-readable speed string.
    pub fn speed_string(&self) -> String {
        format_speed(self.progress.speed_bytes_per_sec)
    }

    /// Returns a human-readable ETA string.
    pub fn eta_string(&self) -> String {
        match self.progress.eta_secs {
            Some(secs) => format_duration(secs as u64),
            None => "Unknown".to_string(),
        }
    }
}

/// Configuration for the download manager.
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// Maximum number of concurrent downloads.
    pub max_concurrent: usize,

    /// Default download directory.
    pub download_dir: PathBuf,

    /// Whether to ask for a save location for each download.
    pub ask_save_location: bool,

    /// Whether to automatically open files after downloading.
    pub auto_open: bool,

    /// Maximum total download speed in bytes/sec (0 = unlimited).
    pub speed_limit_bytes_per_sec: u64,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 3,
            download_dir: dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")),
            ask_save_location: false,
            auto_open: false,
            speed_limit_bytes_per_sec: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Download manager
// ---------------------------------------------------------------------------

/// The main download manager coordinating all download operations.
///
/// # Examples
///
/// ```no_run
/// use nova_features::downloads::DownloadManager;
///
/// let mut manager = DownloadManager::new(Default::default());
/// let download = manager.enqueue("https://example.com/file.zip", "file.zip", None);
/// ```
pub struct DownloadManager {
    /// All downloads keyed by ID.
    downloads: HashMap<String, Download>,

    /// Queue of download IDs in priority order.
    queue: Vec<String>,

    /// Configuration for the manager.
    config: DownloadConfig,

    /// Path to the download history file.
    history_path: Option<PathBuf>,
}

impl DownloadManager {
    /// Creates a new download manager with the given configuration.
    pub fn new(config: DownloadConfig) -> Self {
        info!(
            "Initializing download manager (max concurrent: {})",
            config.max_concurrent
        );
        std::fs::create_dir_all(&config.download_dir).ok();
        Self {
            downloads: HashMap::new(),
            queue: Vec::new(),
            config,
            history_path: None,
        }
    }

    /// Creates a download manager with persistence.
    pub fn with_persistence(config: DownloadConfig, history_path: impl Into<PathBuf>) -> Self {
        let path = history_path.into();
        let mut manager = Self::new(config);
        manager.history_path = Some(path.clone());
        if path.exists() {
            match manager.load_history() {
                Ok(_) => info!("Loaded download history from {:?}", path),
                Err(e) => warn!("Failed to load download history: {}", e),
            }
        }
        manager
    }

    // -----------------------------------------------------------------------
    // Queue management
    // -----------------------------------------------------------------------

    /// Enqueues a new download.
    ///
    /// The download will start automatically if below the concurrency limit.
    pub fn enqueue(
        &mut self,
        url: &str,
        filename: &str,
        referrer: Option<&str>,
    ) -> Result<Download> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let destination = self.config.download_dir.join(filename);

        let download = Download {
            id: id.clone(),
            url: url.to_string(),
            destination: destination.clone(),
            filename: filename.to_string(),
            mime_type: None,
            state: DownloadState::Queued,
            progress: DownloadProgress {
                total_bytes: None,
                downloaded_bytes: 0,
                speed_bytes_per_sec: 0.0,
                eta_secs: None,
                percent: None,
                started_at: now,
                updated_at: now,
            },
            total_bytes: None,
            referrer: referrer.map(|s| s.to_string()),
            priority: 0,
            supports_resume: false,
            error_message: None,
            created_at: now,
            completed_at: None,
        };

        debug!("Enqueued download: {} -> {:?}", url, destination);
        self.downloads.insert(id.clone(), download.clone());
        self.queue.push(id.clone());
        self.maybe_persist()?;
        info!("Download enqueued: {}", filename);
        Ok(download)
    }

    /// Removes a download from the queue and manager.
    pub fn remove(&mut self, id: &str) -> Result<()> {
        let download = self
            .downloads
            .remove(id)
            .ok_or_else(|| DownloadError::NotFound(id.to_string()))?;
        self.queue.retain(|qid| qid != id);

        // Delete partial file if it exists
        if download.state != DownloadState::Completed {
            if download.destination.exists() {
                std::fs::remove_file(&download.destination)?;
                debug!("Removed partial file: {:?}", download.destination);
            }
        }

        self.maybe_persist()?;
        info!("Removed download: {}", id);
        Ok(())
    }

    /// Clears all completed and cancelled downloads from history.
    pub fn clear_completed(&mut self) -> usize {
        let to_remove: Vec<String> = self
            .downloads
            .iter()
            .filter(|(_, d)| d.is_finished())
            .map(|(id, _)| id.clone())
            .collect();
        let count = to_remove.len();
        for id in &to_remove {
            self.downloads.remove(id);
            self.queue.retain(|qid| qid != id);
        }
        let _ = self.maybe_persist();
        info!("Cleared {} completed downloads", count);
        count
    }

    // -----------------------------------------------------------------------
    // State control
    // -----------------------------------------------------------------------

    /// Pauses an active download.
    pub fn pause(&mut self, id: &str) -> Result<()> {
        let download = self
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.to_string()))?;

        if !download.can_pause() {
            return Err(DownloadError::Other(format!(
                "Download {} cannot be paused (state: {:?})",
                id, download.state
            )));
        }

        download.state = DownloadState::Paused;
        download.progress.speed_bytes_per_sec = 0.0;
        download.progress.eta_secs = None;
        download.progress.updated_at = Utc::now();
        info!("Paused download: {}", id);
        self.maybe_persist()?;
        Ok(())
    }

    /// Resumes a paused download.
    pub fn resume(&mut self, id: &str) -> Result<()> {
        let download = self
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.to_string()))?;

        if !download.can_resume() {
            return Err(DownloadError::CannotResume(format!(
                "Download {} cannot be resumed (state: {:?}, supports_resume: {})",
                id, download.state, download.supports_resume
            )));
        }

        download.state = DownloadState::Downloading;
        download.progress.updated_at = Utc::now();
        info!("Resumed download: {}", id);
        self.maybe_persist()?;
        Ok(())
    }

    /// Cancels an active or queued download.
    pub fn cancel(&mut self, id: &str) -> Result<()> {
        let download = self
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.to_string()))?;

        if download.is_finished() {
            return Err(DownloadError::AlreadyCompleted(id.to_string()));
        }

        download.state = DownloadState::Cancelled;
        download.completed_at = Some(Utc::now());
        download.progress.updated_at = Utc::now();
        info!("Cancelled download: {}", id);
        self.maybe_persist()?;
        Ok(())
    }

    /// Retries a failed download.
    pub fn retry(&mut self, id: &str) -> Result<()> {
        let download = self
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.to_string()))?;

        if download.state != DownloadState::Failed {
            return Err(DownloadError::Other(format!(
                "Download {} is not in a failed state",
                id
            )));
        }

        download.state = DownloadState::Queued;
        download.progress.downloaded_bytes = 0;
        download.progress.speed_bytes_per_sec = 0.0;
        download.progress.eta_secs = None;
        download.progress.percent = None;
        download.error_message = None;
        download.progress.started_at = Utc::now();
        download.progress.updated_at = Utc::now();
        self.queue.push(id.to_string());
        info!("Retrying download: {}", id);
        self.maybe_persist()?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Progress tracking
    // -----------------------------------------------------------------------

    /// Updates the progress of an active download.
    ///
    /// This is called periodically by the download worker to report progress.
    pub fn update_progress(
        &mut self,
        id: &str,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        speed_bytes_per_sec: f64,
    ) -> Result<()> {
        let download = self
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.to_string()))?;

        let now = Utc::now();
        download.progress.downloaded_bytes = downloaded_bytes;
        download.progress.speed_bytes_per_sec = speed_bytes_per_sec;
        download.progress.updated_at = now;

        let effective_total = total_bytes.or(download.total_bytes);
        download.progress.total_bytes = effective_total;

        if let Some(total) = effective_total {
            if total > 0 {
                let percent = (downloaded_bytes as f64 / total as f64) * 100.0;
                download.progress.percent = Some(percent.min(100.0));

                let remaining = total.saturating_sub(downloaded_bytes);
                if speed_bytes_per_sec > 0.0 {
                    download.progress.eta_secs = Some(remaining as f64 / speed_bytes_per_sec);
                }
            }
        }

        // Check if download is complete
        if let Some(total) = effective_total {
            if downloaded_bytes >= total {
                download.state = DownloadState::Completed;
                download.completed_at = Some(now);
                download.progress.percent = Some(100.0);
                download.progress.speed_bytes_per_sec = 0.0;
                download.progress.eta_secs = None;
                info!("Download completed: {} ({})", download.filename, download.id);
            }
        }

        self.maybe_persist()?;
        Ok(())
    }

    /// Marks a download as failed with an error message.
    pub fn mark_failed(&mut self, id: &str, error: &str) -> Result<()> {
        let download = self
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.to_string()))?;

        download.state = DownloadState::Failed;
        download.error_message = Some(error.to_string());
        download.completed_at = Some(Utc::now());
        download.progress.updated_at = Utc::now();
        error!("Download failed: {} - {}", id, error);
        self.maybe_persist()?;
        Ok(())
    }

    /// Marks a download as actively downloading.
    pub fn mark_downloading(&mut self, id: &str, total_bytes: Option<u64>, supports_resume: bool) -> Result<()> {
        let download = self
            .downloads
            .get_mut(id)
            .ok_or_else(|| DownloadError::NotFound(id.to_string()))?;

        download.state = DownloadState::Downloading;
        download.total_bytes = total_bytes;
        download.supports_resume = supports_resume;
        download.progress.total_bytes = total_bytes;
        download.progress.started_at = Utc::now();
        download.progress.updated_at = Utc::now();
        self.maybe_persist()?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Querying
    // -----------------------------------------------------------------------

    /// Returns a reference to a download by ID.
    pub fn get(&self, id: &str) -> Option<&Download> {
        self.downloads.get(id)
    }

    /// Returns all downloads.
    pub fn all(&self) -> Vec<&Download> {
        self.downloads.values().collect()
    }

    /// Returns active downloads (queued or downloading).
    pub fn active(&self) -> Vec<&Download> {
        self.downloads
            .values()
            .filter(|d| {
                d.state == DownloadState::Downloading || d.state == DownloadState::Queued
            })
            .collect()
    }

    /// Returns completed downloads.
    pub fn completed(&self) -> Vec<&Download> {
        self.downloads
            .values()
            .filter(|d| d.state == DownloadState::Completed)
            .collect()
    }

    /// Returns failed downloads.
    pub fn failed(&self) -> Vec<&Download> {
        self.downloads
            .values()
            .filter(|d| d.state == DownloadState::Failed)
            .collect()
    }

    /// Returns the number of active downloads.
    pub fn active_count(&self) -> usize {
        self.active().len()
    }

    /// Returns the number of downloads below the concurrency limit.
    pub fn available_slots(&self) -> usize {
        let active = self
            .downloads
            .values()
            .filter(|d| d.state == DownloadState::Downloading)
            .count();
        self.config.max_concurrent.saturating_sub(active)
    }

    /// Returns the next queued download to start, considering priority.
    pub fn next_queued(&self) -> Option<&Download> {
        self.queue
            .iter()
            .filter_map(|id| self.downloads.get(id))
            .filter(|d| d.state == DownloadState::Queued)
            .max_by_key(|d| d.priority)
    }

    /// Searches downloads by URL, filename, or referrer.
    pub fn search(&self, query: &str) -> Vec<&Download> {
        let query = query.to_lowercase();
        self.downloads
            .values()
            .filter(|d| {
                d.url.to_lowercase().contains(&query)
                    || d.filename.to_lowercase().contains(&query)
                    || d.referrer
                        .as_ref()
                        .map(|r| r.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Returns the total number of downloads.
    pub fn len(&self) -> usize {
        self.downloads.len()
    }

    /// Returns true if there are no downloads.
    pub fn is_empty(&self) -> bool {
        self.downloads.is_empty()
    }

    // -----------------------------------------------------------------------
    // Configuration
    // -----------------------------------------------------------------------

    /// Returns the current configuration.
    pub fn config(&self) -> &DownloadConfig {
        &self.config
    }

    /// Updates the download directory.
    pub fn set_download_dir(&mut self, dir: impl Into<PathBuf>) {
        self.config.download_dir = dir.into();
        std::fs::create_dir_all(&self.config.download_dir).ok();
        info!("Download directory set to {:?}", self.config.download_dir);
    }

    /// Sets the maximum number of concurrent downloads.
    pub fn set_max_concurrent(&mut self, max: usize) {
        self.config.max_concurrent = max;
        info!("Max concurrent downloads set to {}", max);
    }

    /// Sets the speed limit in bytes per second.
    pub fn set_speed_limit(&mut self, bytes_per_sec: u64) {
        self.config.speed_limit_bytes_per_sec = bytes_per_sec;
        info!("Speed limit set to {} bytes/sec", bytes_per_sec);
    }

    // -----------------------------------------------------------------------
    // Persistence
    // -----------------------------------------------------------------------

    /// Saves download history to a file.
    pub fn save_history_to(&self, path: &Path) -> Result<()> {
        let downloads: Vec<&Download> = self.downloads.values().collect();
        let json = serde_json::to_string_pretty(&downloads)?;
        std::fs::write(path, json)?;
        info!("Saved {} downloads to {:?}", downloads.len(), path);
        Ok(())
    }

    /// Loads download history from a file.
    fn load_history(&mut self) -> Result<()> {
        if let Some(ref path) = self.history_path {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                let loaded: Vec<Download> = serde_json::from_str(&content)?;
                for download in loaded {
                    self.downloads.insert(download.id.clone(), download);
                }
                info!("Loaded {} downloads from history", self.downloads.len());
            }
        }
        Ok(())
    }

    /// Persists to the configured history path.
    fn maybe_persist(&self) -> Result<()> {
        if let Some(ref path) = self.history_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let downloads: Vec<&Download> = self.downloads.values().collect();
            let json = serde_json::to_string_pretty(&downloads)?;
            std::fs::write(path, json)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Formats a speed in bytes/sec to a human-readable string.
fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec < 1024.0 {
        format!("{:.0} B/s", bytes_per_sec)
    } else if bytes_per_sec < 1024.0 * 1024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1024.0)
    } else if bytes_per_sec < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB/s", bytes_per_sec / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB/s", bytes_per_sec / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Formats a duration in seconds to a human-readable string.
fn format_duration(total_secs: u64) -> String {
    if total_secs < 60 {
        format!("{}s", total_secs)
    } else if total_secs < 3600 {
        format!("{}m {}s", total_secs / 60, total_secs % 60)
    } else {
        format!(
            "{}h {}m {}s",
            total_secs / 3600,
            (total_secs % 3600) / 60,
            total_secs % 60
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_and_get() {
        let mut mgr = DownloadManager::new(DownloadConfig::default());
        let dl = mgr
            .enqueue("https://example.com/file.zip", "file.zip", None)
            .unwrap();
        assert_eq!(dl.url, "https://example.com/file.zip");
        assert_eq!(dl.state, DownloadState::Queued);
        assert!(mgr.get(&dl.id).is_some());
    }

    #[test]
    fn test_pause_resume_cancel() {
        let mut mgr = DownloadManager::new(DownloadConfig::default());
        let dl = mgr
            .enqueue("https://example.com/file.zip", "file.zip", None)
            .unwrap();

        mgr.mark_downloading(&dl.id, Some(1024 * 1024), true).unwrap();
        mgr.pause(&dl.id).unwrap();
        assert_eq!(mgr.get(&dl.id).unwrap().state, DownloadState::Paused);

        mgr.resume(&dl.id).unwrap();
        assert_eq!(mgr.get(&dl.id).unwrap().state, DownloadState::Downloading);

        mgr.cancel(&dl.id).unwrap();
        assert_eq!(mgr.get(&dl.id).unwrap().state, DownloadState::Cancelled);
    }

    #[test]
    fn test_progress_update() {
        let mut mgr = DownloadManager::new(DownloadConfig::default());
        let dl = mgr
            .enqueue("https://example.com/file.zip", "file.zip", None)
            .unwrap();

        mgr.mark_downloading(&dl.id, Some(1000), true).unwrap();
        mgr.update_progress(&dl.id, 500, Some(1000), 1024.0)
            .unwrap();

        let updated = mgr.get(&dl.id).unwrap();
        assert_eq!(updated.progress.downloaded_bytes, 500);
        assert!(updated.progress.percent.unwrap() > 49.0);
    }

    #[test]
    fn test_completion() {
        let mut mgr = DownloadManager::new(DownloadConfig::default());
        let dl = mgr
            .enqueue("https://example.com/file.zip", "file.zip", None)
            .unwrap();

        mgr.mark_downloading(&dl.id, Some(1000), true).unwrap();
        mgr.update_progress(&dl.id, 1000, Some(1000), 0.0)
            .unwrap();

        let completed = mgr.get(&dl.id).unwrap();
        assert_eq!(completed.state, DownloadState::Completed);
        assert_eq!(completed.progress.percent, Some(100.0));
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(500.0), "500 B/s");
        assert_eq!(format_speed(2048.0), "2.0 KB/s");
        assert_eq!(format_speed(5_000_000.0), "4.8 MB/s");
    }
}