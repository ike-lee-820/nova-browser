//! # Nova Developer Tools
//!
//! A comprehensive suite of developer tools including element inspection,
//! network monitoring, and performance profiling.
//!
//! ## Features
//!
//! - **Element inspector**: DOM tree navigation, CSS inspection, box model visualization
//! - **Network monitor**: Request/response tracking, timing breakdown, filtering
//! - **Performance profiler**: JavaScript CPU profiling, memory heap snapshots
//! - **Console**: Log collection and filtering with severity levels
//! - **Timeline**: Frame rendering, layout, and paint event tracking
//!
//! ## Architecture
//!
//! The [`DevTools`] struct serves as the central coordinator for all developer
//! tool modules. Each module maintains its own state and is accessed through
//! the main [`DevTools`] API.

use chrono::{DateTime, Utc};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur during developer tools operations.
#[derive(Error, Debug)]
pub enum DevToolsError {
    /// A panel or tool was not found.
    #[error("tool not found: {0}")]
    NotFound(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// A generic error.
    #[error("dev tools error: {0}")]
    Other(String),
}

/// Convenience type alias.
pub type Result<T> = std::result::Result<T, DevToolsError>;

// ===========================================================================
// Element Inspector
// ===========================================================================

/// A DOM node representation for the inspector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomNode {
    /// Unique node identifier.
    pub id: u64,

    /// Tag name (e.g., "div", "span", "p").
    pub tag_name: String,

    /// Node type (1 = element, 3 = text, 8 = comment).
    pub node_type: u8,

    /// Text content for text nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,

    /// CSS attributes applied to this node.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,

    /// Computed CSS styles.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub computed_styles: HashMap<String, String>,

    /// Child node IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<u64>,

    /// Parent node ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<u64>,

    /// Box model dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub box_model: Option<BoxModel>,
}

/// CSS box model dimensions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoxModel {
    /// Content area width.
    pub content_width: f64,
    /// Content area height.
    pub content_height: f64,
    /// Padding top.
    pub padding_top: f64,
    /// Padding right.
    pub padding_right: f64,
    /// Padding bottom.
    pub padding_bottom: f64,
    /// Padding left.
    pub padding_left: f64,
    /// Border top.
    pub border_top: f64,
    /// Border right.
    pub border_right: f64,
    /// Border bottom.
    pub border_bottom: f64,
    /// Border left.
    pub border_left: f64,
    /// Margin top.
    pub margin_top: f64,
    /// Margin right.
    pub margin_right: f64,
    /// Margin bottom.
    pub margin_bottom: f64,
    /// Margin left.
    pub margin_left: f64,
}

/// The element inspector state.
pub struct ElementInspector {
    /// The DOM tree, keyed by node ID.
    dom_tree: HashMap<u64, DomNode>,

    /// The currently selected node ID.
    selected_node: Option<u64>,

    /// Breadcrumb path from root to selected node.
    breadcrumbs: Vec<u64>,

    /// Whether the inspector is active.
    active: bool,
}

impl ElementInspector {
    /// Creates a new element inspector.
    pub fn new() -> Self {
        Self {
            dom_tree: HashMap::new(),
            selected_node: None,
            breadcrumbs: Vec::new(),
            active: false,
        }
    }

    /// Activates the inspector.
    pub fn activate(&mut self) {
        self.active = true;
        info!("Element inspector activated");
    }

    /// Deactivates the inspector.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.selected_node = None;
        self.breadcrumbs.clear();
        info!("Element inspector deactivated");
    }

    /// Returns whether the inspector is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Loads a DOM tree snapshot.
    pub fn load_dom(&mut self, nodes: Vec<DomNode>) {
        self.dom_tree.clear();
        for node in nodes {
            self.dom_tree.insert(node.id, node);
        }
        debug!("Loaded {} DOM nodes", self.dom_tree.len());
    }

    /// Selects a node by ID.
    pub fn select_node(&mut self, node_id: u64) {
        self.selected_node = Some(node_id);
        self.build_breadcrumbs(node_id);
        debug!("Selected node: {}", node_id);
    }

    /// Returns the selected node.
    pub fn selected_node(&self) -> Option<&DomNode> {
        self.selected_node
            .and_then(|id| self.dom_tree.get(&id))
    }

    /// Returns the breadcrumb path.
    pub fn breadcrumbs(&self) -> &[u64] {
        &self.breadcrumbs
    }

    /// Returns a node by ID.
    pub fn get_node(&self, node_id: u64) -> Option<&DomNode> {
        self.dom_tree.get(&node_id)
    }

    /// Returns all nodes with a given tag name.
    pub fn find_by_tag(&self, tag_name: &str) -> Vec<&DomNode> {
        self.dom_tree
            .values()
            .filter(|n| n.tag_name.eq_ignore_ascii_case(tag_name))
            .collect()
    }

    /// Returns all nodes matching a CSS selector (simplified).
    pub fn find_by_selector(&self, selector: &str) -> Vec<&DomNode> {
        // Simplified selector matching: support tag, class, and id
        let selector = selector.trim();
        let mut results = Vec::new();

        for node in self.dom_tree.values() {
            if selector.starts_with('#') {
                // ID selector
                let id = &selector[1..];
                if node.attributes.get("id").map(|v| v == id).unwrap_or(false) {
                    results.push(node);
                }
            } else if selector.starts_with('.') {
                // Class selector
                let class = &selector[1..];
                if node
                    .attributes
                    .get("class")
                    .map(|v| v.split_whitespace().any(|c| c == class))
                    .unwrap_or(false)
                {
                    results.push(node);
                }
            } else if node.tag_name.eq_ignore_ascii_case(selector) {
                results.push(node);
            }
        }

        results
    }

    /// Builds the breadcrumb path from root to the given node.
    fn build_breadcrumbs(&mut self, node_id: u64) {
        self.breadcrumbs.clear();
        let mut current = node_id;
        let mut path = Vec::new();
        // Safety limit to prevent infinite loops
        for _ in 0..100 {
            path.push(current);
            if let Some(node) = self.dom_tree.get(&current) {
                match node.parent_id {
                    Some(pid) => current = pid,
                    None => break,
                }
            } else {
                break;
            }
        }
        path.reverse();
        self.breadcrumbs = path;
    }
}

impl Default for ElementInspector {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Network Monitor
// ===========================================================================

/// HTTP method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

/// A single network request/response entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEntry {
    /// Unique identifier.
    pub id: String,

    /// The full request URL.
    pub url: String,

    /// HTTP method.
    pub method: HttpMethod,

    /// HTTP status code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,

    /// Request headers.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub request_headers: HashMap<String, String>,

    /// Response headers.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub response_headers: HashMap<String, String>,

    /// Request body size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body_size: Option<u64>,

    /// Response body size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_body_size: Option<u64>,

    /// Timing breakdown in milliseconds.
    pub timing: RequestTiming,

    /// Whether this was a third-party request.
    #[serde(default)]
    pub is_third_party: bool,

    /// The resource type (e.g., "script", "stylesheet", "image", "xhr").
    pub resource_type: String,

    /// The initiator (page URL that triggered the request).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initiator: Option<String>,

    /// When the request was made.
    pub timestamp: DateTime<Utc>,
}

/// Timing breakdown for a network request.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RequestTiming {
    /// DNS lookup time in ms.
    #[serde(default)]
    pub dns_lookup_ms: f64,
    /// TCP connection time in ms.
    #[serde(default)]
    pub tcp_connect_ms: f64,
    /// TLS handshake time in ms.
    #[serde(default)]
    pub tls_handshake_ms: f64,
    /// Time to first byte in ms.
    #[serde(default)]
    pub ttfb_ms: f64,
    /// Content download time in ms.
    #[serde(default)]
    pub content_download_ms: f64,
    /// Total request time in ms.
    #[serde(default)]
    pub total_ms: f64,
}

/// Network monitor state.
pub struct NetworkMonitor {
    /// All recorded network entries.
    entries: VecDeque<NetworkEntry>,

    /// Maximum number of entries to retain.
    max_entries: usize,

    /// Whether the monitor is recording.
    recording: bool,

    /// Active filters.
    filters: Vec<NetworkFilter>,
}

/// Filter for network entries.
#[derive(Debug, Clone)]
pub enum NetworkFilter {
    /// Filter by resource type.
    ResourceType(String),
    /// Filter by HTTP status code category.
    StatusCategory(StatusCategory),
    /// Filter by URL pattern.
    UrlPattern(String),
    /// Only show third-party requests.
    ThirdParty,
    /// Only show requests with errors (4xx, 5xx).
    Errors,
}

/// HTTP status code categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCategory {
    /// 1xx: Informational.
    Info,
    /// 2xx: Success.
    Success,
    /// 3xx: Redirection.
    Redirect,
    /// 4xx: Client error.
    ClientError,
    /// 5xx: Server error.
    ServerError,
}

impl NetworkMonitor {
    /// Creates a new network monitor.
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: 5000,
            recording: false,
            filters: Vec::new(),
        }
    }

    /// Starts recording network requests.
    pub fn start_recording(&mut self) {
        self.recording = true;
        info!("Network monitor started recording");
    }

    /// Stops recording network requests.
    pub fn stop_recording(&mut self) {
        self.recording = false;
        info!("Network monitor stopped recording");
    }

    /// Returns whether the monitor is recording.
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Records a network request.
    pub fn record_request(&mut self, entry: NetworkEntry) {
        if !self.recording {
            return;
        }
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Returns all recorded entries, newest first.
    pub fn entries(&self) -> Vec<&NetworkEntry> {
        let mut entries: Vec<&NetworkEntry> = self.entries.iter().collect();
        entries.reverse();
        entries
    }

    /// Returns filtered entries.
    pub fn filtered_entries(&self) -> Vec<&NetworkEntry> {
        let mut entries: Vec<&NetworkEntry> = self.entries.iter().collect();

        for filter in &self.filters {
            entries.retain(|entry| match filter {
                NetworkFilter::ResourceType(rt) => entry.resource_type == *rt,
                NetworkFilter::StatusCategory(sc) => {
                    if let Some(code) = entry.status_code {
                        match sc {
                            StatusCategory::Info => (100..200).contains(&code),
                            StatusCategory::Success => (200..300).contains(&code),
                            StatusCategory::Redirect => (300..400).contains(&code),
                            StatusCategory::ClientError => (400..500).contains(&code),
                            StatusCategory::ServerError => (500..600).contains(&code),
                        }
                    } else {
                        false
                    }
                }
                NetworkFilter::UrlPattern(pattern) => {
                    entry.url.to_lowercase().contains(&pattern.to_lowercase())
                }
                NetworkFilter::ThirdParty => entry.is_third_party,
                NetworkFilter::Errors => {
                    entry
                        .status_code
                        .map(|c| c >= 400)
                        .unwrap_or(true)
                }
            });
        }

        entries.reverse();
        entries
    }

    /// Adds a filter.
    pub fn add_filter(&mut self, filter: NetworkFilter) {
        self.filters.push(filter);
    }

    /// Clears all filters.
    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }

    /// Clears all recorded entries.
    pub fn clear_entries(&mut self) {
        self.entries.clear();
        info!("Network monitor entries cleared");
    }

    /// Returns the total number of entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns summary statistics for the recorded entries.
    pub fn statistics(&self) -> NetworkStats {
        let total = self.entries.len();
        let total_size: u64 = self
            .entries
            .iter()
            .filter_map(|e| e.response_body_size)
            .sum();
        let total_time_ms: f64 = self.entries.iter().map(|e| e.timing.total_ms).sum();
        let errors = self
            .entries
            .iter()
            .filter(|e| e.status_code.map(|c| c >= 400).unwrap_or(false))
            .count();

        NetworkStats {
            total_requests: total,
            total_size_bytes: total_size,
            total_time_ms,
            error_count: errors,
            avg_time_ms: if total > 0 {
                total_time_ms / total as f64
            } else {
                0.0
            },
        }
    }
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Network monitoring statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Total number of requests.
    pub total_requests: usize,
    /// Total response size in bytes.
    pub total_size_bytes: u64,
    /// Total time spent in ms.
    pub total_time_ms: f64,
    /// Number of error responses.
    pub error_count: usize,
    /// Average request time in ms.
    pub avg_time_ms: f64,
}

// ===========================================================================
// Performance Profiler
// ===========================================================================

/// A single performance profile entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileEntry {
    /// Function name or event label.
    pub name: String,

    /// Start time relative to profile start in microseconds.
    pub start_time_us: u64,

    /// Duration in microseconds.
    pub duration_us: u64,

    /// Source file location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Line number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,

    /// Child entries (call tree).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ProfileEntry>,
}

/// A memory heap snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapSnapshot {
    /// Unique identifier.
    pub id: String,

    /// Timestamp when the snapshot was taken.
    pub timestamp: DateTime<Utc>,

    /// Total heap size in bytes.
    pub total_heap_size: u64,

    /// Used heap size in bytes.
    pub used_heap_size: u64,

    /// Number of live objects.
    pub object_count: u64,

    /// Breakdown by object type.
    pub type_breakdown: HashMap<String, HeapTypeInfo>,
}

/// Memory usage by object type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapTypeInfo {
    /// Number of objects of this type.
    pub count: u64,
    /// Total size in bytes.
    pub size_bytes: u64,
}

/// Performance profiler state.
pub struct PerformanceProfiler {
    /// Whether profiling is active.
    profiling: bool,

    /// Recorded profile entries.
    entries: Vec<ProfileEntry>,

    /// Heap snapshots.
    heap_snapshots: Vec<HeapSnapshot>,

    /// Profile start time.
    start_time: Option<DateTime<Utc>>,
}

impl PerformanceProfiler {
    /// Creates a new performance profiler.
    pub fn new() -> Self {
        Self {
            profiling: false,
            entries: Vec::new(),
            heap_snapshots: Vec::new(),
            start_time: None,
        }
    }

    /// Starts profiling.
    pub fn start_profiling(&mut self) {
        self.profiling = true;
        self.start_time = Some(Utc::now());
        self.entries.clear();
        info!("Performance profiler started");
    }

    /// Stops profiling.
    pub fn stop_profiling(&mut self) {
        self.profiling = false;
        info!("Performance profiler stopped");
    }

    /// Returns whether profiling is active.
    pub fn is_profiling(&self) -> bool {
        self.profiling
    }

    /// Records a profile entry.
    pub fn record(&mut self, entry: ProfileEntry) {
        if self.profiling {
            self.entries.push(entry);
        }
    }

    /// Returns all profile entries.
    pub fn entries(&self) -> &[ProfileEntry] {
        &self.entries
    }

    /// Returns the top N entries by duration.
    pub fn top_entries(&self, n: usize) -> Vec<&ProfileEntry> {
        let mut sorted: Vec<&ProfileEntry> = self.entries.iter().collect();
        sorted.sort_by(|a, b| b.duration_us.cmp(&a.duration_us));
        sorted.truncate(n);
        sorted
    }

    /// Returns the total profiling duration in microseconds.
    pub fn total_duration_us(&self) -> u64 {
        self.entries.iter().map(|e| e.duration_us).sum()
    }

    /// Takes a heap snapshot.
    pub fn take_heap_snapshot(
        &mut self,
        total_heap_size: u64,
        used_heap_size: u64,
        object_count: u64,
        type_breakdown: HashMap<String, HeapTypeInfo>,
    ) -> &HeapSnapshot {
        let snapshot = HeapSnapshot {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            total_heap_size,
            used_heap_size,
            object_count,
            type_breakdown,
        };
        self.heap_snapshots.push(snapshot);
        debug!(
            "Heap snapshot taken: {} used / {} total",
            used_heap_size, total_heap_size
        );
        self.heap_snapshots.last().unwrap()
    }

    /// Returns all heap snapshots.
    pub fn heap_snapshots(&self) -> &[HeapSnapshot] {
        &self.heap_snapshots
    }

    /// Compares two heap snapshots and returns the diff.
    pub fn compare_snapshots(
        &self,
        id1: &str,
        id2: &str,
    ) -> Result<HeapSnapshotDiff> {
        let s1 = self
            .heap_snapshots
            .iter()
            .find(|s| s.id == id1)
            .ok_or_else(|| DevToolsError::NotFound(id1.to_string()))?;
        let s2 = self
            .heap_snapshots
            .iter()
            .find(|s| s.id == id2)
            .ok_or_else(|| DevToolsError::NotFound(id2.to_string()))?;

        Ok(HeapSnapshotDiff {
            heap_size_diff: s2.used_heap_size as i64 - s1.used_heap_size as i64,
            object_count_diff: s2.object_count as i64 - s1.object_count as i64,
        })
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Diff between two heap snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapSnapshotDiff {
    /// Difference in heap size (positive = growth).
    pub heap_size_diff: i64,
    /// Difference in object count (positive = growth).
    pub object_count_diff: i64,
}

// ===========================================================================
// Console
// ===========================================================================

/// Severity level for console messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsoleLevel {
    Log,
    Info,
    Warn,
    Error,
    Debug,
}

/// A console message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    /// Unique identifier.
    pub id: String,

    /// Severity level.
    pub level: ConsoleLevel,

    /// The message text.
    pub message: String,

    /// Source file or URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Line number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,

    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

/// Console message collector.
pub struct Console {
    /// All collected messages.
    messages: VecDeque<ConsoleMessage>,

    /// Maximum number of messages to retain.
    max_messages: usize,

    /// Active level filters.
    level_filter: Vec<ConsoleLevel>,
}

impl Console {
    /// Creates a new console collector.
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages: 1000,
            level_filter: vec![
                ConsoleLevel::Log,
                ConsoleLevel::Info,
                ConsoleLevel::Warn,
                ConsoleLevel::Error,
                ConsoleLevel::Debug,
            ],
        }
    }

    /// Logs a message.
    pub fn log(&mut self, level: ConsoleLevel, message: &str) {
        let msg = ConsoleMessage {
            id: Uuid::new_v4().to_string(),
            level,
            message: message.to_string(),
            source: None,
            line: None,
            timestamp: Utc::now(),
        };
        if self.messages.len() >= self.max_messages {
            self.messages.pop_front();
        }
        self.messages.push_back(msg);
    }

    /// Returns all messages matching the current filters.
    pub fn messages(&self) -> Vec<&ConsoleMessage> {
        self.messages
            .iter()
            .filter(|m| self.level_filter.contains(&m.level))
            .collect()
    }

    /// Sets the level filter.
    pub fn set_level_filter(&mut self, levels: Vec<ConsoleLevel>) {
        self.level_filter = levels;
    }

    /// Clears all messages.
    pub fn clear(&mut self) {
        self.messages.clear();
        info!("Console cleared");
    }

    /// Returns the number of messages.
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Main DevTools
// ===========================================================================

/// The main developer tools coordinator.
pub struct DevTools {
    /// Element inspector.
    pub inspector: ElementInspector,

    /// Network monitor.
    pub network: NetworkMonitor,

    /// Performance profiler.
    pub profiler: PerformanceProfiler,

    /// Console.
    pub console: Console,

    /// Whether dev tools is open.
    open: bool,

    /// The currently active panel.
    active_panel: DevToolsPanel,
}

/// Available developer tools panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevToolsPanel {
    Elements,
    Console,
    Network,
    Performance,
    Memory,
    Application,
    Security,
    Audit,
}

impl DevTools {
    /// Creates a new DevTools instance.
    pub fn new() -> Self {
        info!("Developer tools initialized");
        Self {
            inspector: ElementInspector::new(),
            network: NetworkMonitor::new(),
            profiler: PerformanceProfiler::new(),
            console: Console::new(),
            open: false,
            active_panel: DevToolsPanel::Elements,
        }
    }

    /// Opens the developer tools.
    pub fn open(&mut self) {
        self.open = true;
        info!("Developer tools opened");
    }

    /// Closes the developer tools.
    pub fn close(&mut self) {
        self.open = false;
        info!("Developer tools closed");
    }

    /// Toggles the developer tools.
    pub fn toggle(&mut self) -> bool {
        if self.open {
            self.close();
        } else {
            self.open();
        }
        self.open
    }

    /// Returns whether developer tools is open.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Sets the active panel.
    pub fn set_active_panel(&mut self, panel: DevToolsPanel) {
        self.active_panel = panel;
        debug!("Active panel: {:?}", panel);
    }

    /// Returns the active panel.
    pub fn active_panel(&self) -> DevToolsPanel {
        self.active_panel
    }
}

impl Default for DevTools {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_inspector() {
        let mut inspector = ElementInspector::new();
        inspector.activate();
        assert!(inspector.is_active());

        let node = DomNode {
            id: 1,
            tag_name: "div".into(),
            node_type: 1,
            text_content: None,
            attributes: {
                let mut m = HashMap::new();
                m.insert("class".into(), "container".into());
                m
            },
            computed_styles: HashMap::new(),
            children: vec![],
            parent_id: None,
            box_model: None,
        };

        inspector.load_dom(vec![node]);
        inspector.select_node(1);
        assert!(inspector.selected_node().is_some());
    }

    #[test]
    fn test_network_monitor() {
        let mut monitor = NetworkMonitor::new();
        monitor.start_recording();
        assert!(monitor.is_recording());

        let entry = NetworkEntry {
            id: "1".into(),
            url: "https://example.com/api".into(),
            method: HttpMethod::GET,
            status_code: Some(200),
            request_headers: HashMap::new(),
            response_headers: HashMap::new(),
            request_body_size: None,
            response_body_size: Some(1024),
            timing: RequestTiming {
                dns_lookup_ms: 5.0,
                tcp_connect_ms: 10.0,
                tls_handshake_ms: 15.0,
                ttfb_ms: 50.0,
                content_download_ms: 100.0,
                total_ms: 180.0,
            },
            is_third_party: false,
            resource_type: "xhr".into(),
            initiator: None,
            timestamp: Utc::now(),
        };

        monitor.record_request(entry);
        assert_eq!(monitor.entry_count(), 1);
    }

    #[test]
    fn test_performance_profiler() {
        let mut profiler = PerformanceProfiler::new();
        profiler.start_profiling();
        assert!(profiler.is_profiling());

        let entry = ProfileEntry {
            name: "render".into(),
            start_time_us: 0,
            duration_us: 5000,
            source: None,
            line: None,
            children: vec![],
        };

        profiler.record(entry);
        assert_eq!(profiler.entries().len(), 1);
    }

    #[test]
    fn test_console() {
        let mut console = Console::new();
        console.log(ConsoleLevel::Info, "Test message");
        console.log(ConsoleLevel::Error, "Error message");
        assert_eq!(console.message_count(), 2);
    }

    #[test]
    fn test_dev_tools_toggle() {
        let mut tools = DevTools::new();
        assert!(!tools.is_open());
        tools.open();
        assert!(tools.is_open());
        tools.close();
        assert!(!tools.is_open());
    }
}