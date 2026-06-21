//! # Nova Ad Blocker
//!
//! A high-performance ad and tracker blocking engine with filter list
//! support, custom rules, and anti-phishing protection.
//!
//! ## Features
//!
//! - **Filter lists**: Support for EasyList, EasyPrivacy, and custom lists
//! - **Custom rules**: User-defined allow/block rules with pattern matching
//! - **Anti-phishing**: Google Safe Browsing-style URL checking
//! - **Element hiding**: CSS-based element hiding rules
//! - **Network filtering**: URL pattern matching for request blocking
//! - **Whitelist**: Per-domain and per-page allowlisting
//!
//! ## Architecture
//!
//! The [`AdBlocker`] maintains an in-memory rule database built from filter
//! lists. Rules are compiled into efficient data structures for fast matching.
//! The engine supports both network-level filtering (blocking requests) and
//! cosmetic filtering (hiding page elements via CSS).

use log::{debug, error, info, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur during ad blocker operations.
#[derive(Error, Debug)]
pub enum AdBlockerError {
    /// Failed to load a filter list.
    #[error("failed to load filter list: {0}")]
    FilterListError(String),

    /// A rule has invalid syntax.
    #[error("invalid rule syntax: {0}")]
    InvalidRule(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Regex compilation error.
    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),

    /// A generic error.
    #[error("ad blocker error: {0}")]
    Other(String),
}

/// Convenience type alias.
pub type Result<T> = std::result::Result<T, AdBlockerError>;

// ---------------------------------------------------------------------------
// Rule types
// ---------------------------------------------------------------------------

/// The type of a blocking rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    /// Block a network request.
    Block,
    /// Allow a network request (whitelist).
    Allow,
    /// Hide an element via CSS.
    ElementHide,
    /// Apply a CSS style rule.
    ElementStyle,
    /// Block a script.
    Script,
    /// Block an image.
    Image,
    /// Block a third-party request.
    ThirdParty,
    /// Block a first-party request.
    FirstParty,
    /// A phishing/malware domain.
    Malware,
}

/// A parsed filter rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// The raw rule text.
    pub raw: String,

    /// The type of rule.
    pub rule_type: RuleType,

    /// The domain this rule applies to (if domain-specific).
    pub domain: Option<String>,

    /// The URL pattern to match.
    pub pattern: String,

    /// Compiled regex for URL matching.
    #[serde(skip, default = "default_regex")]
    pub regex: Regex,

    /// Whether this rule is enabled.
    pub enabled: bool,

    /// CSS selector (for element hiding rules).
    pub selector: Option<String>,
}

/// Default regex used when deserializing.
fn default_regex() -> Regex {
    Regex::new(".*").unwrap()
}

/// A filter list source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterList {
    /// Unique identifier.
    pub id: String,

    /// Human-readable name.
    pub name: String,

    /// URL to fetch the list from.
    pub url: Option<String>,

    /// Description of the list.
    pub description: String,

    /// Number of rules in this list.
    pub rule_count: usize,

    /// Whether this list is enabled.
    pub enabled: bool,

    /// When the list was last updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

/// A user-defined custom rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    /// Unique identifier.
    pub id: String,

    /// The rule text.
    pub rule: String,

    /// Whether this rule is enabled.
    pub enabled: bool,

    /// Description of what this rule does.
    pub description: String,
}

/// Statistics about the ad blocker's activity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockingStats {
    /// Total requests blocked.
    pub total_blocked: u64,

    /// Total elements hidden.
    pub total_hidden: u64,

    /// Total requests allowed.
    pub total_allowed: u64,

    /// Phishing/malware sites blocked.
    pub phishing_blocked: u64,
}

/// Configuration for the ad blocker.
#[derive(Debug, Clone)]
pub struct AdBlockerConfig {
    /// Whether the ad blocker is enabled.
    pub enabled: bool,

    /// Whether to block trackers.
    pub block_trackers: bool,

    /// Whether to block social media widgets.
    pub block_social: bool,

    /// Whether to apply cosmetic filtering.
    pub cosmetic_filtering: bool,

    /// Whether to enable anti-phishing.
    pub anti_phishing: bool,

    /// Domains that are whitelisted (ad blocking disabled).
    pub whitelisted_domains: HashSet<String>,
}

impl Default for AdBlockerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            block_trackers: true,
            block_social: true,
            cosmetic_filtering: true,
            anti_phishing: true,
            whitelisted_domains: HashSet::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Ad blocker
// ---------------------------------------------------------------------------

/// The main ad blocker engine.
///
/// # Examples
///
/// ```no_run
/// use nova_features::ad_blocker::AdBlocker;
///
/// let mut blocker = AdBlocker::new(Default::default());
/// blocker.load_easylist("# EasyList rules...").unwrap();
/// let should_block = blocker.should_block("https://example.com/ad.js", "example.com", true);
/// ```
pub struct AdBlocker {
    config: AdBlockerConfig,

    /// All compiled filter rules.
    rules: Vec<FilterRule>,

    /// Element hiding rules keyed by domain.
    element_hiding_rules: HashMap<String, Vec<String>>,

    /// Known phishing/malware domains.
    phishing_domains: HashSet<String>,

    /// Installed filter lists.
    filter_lists: Vec<FilterList>,

    /// User-defined custom rules.
    custom_rules: Vec<CustomRule>,

    /// Blocking statistics.
    stats: BlockingStats,
}

impl AdBlocker {
    /// Creates a new ad blocker with the given configuration.
    pub fn new(config: AdBlockerConfig) -> Self {
        info!("Ad blocker initialized (enabled: {})", config.enabled);
        Self {
            config,
            rules: Vec::new(),
            element_hiding_rules: HashMap::new(),
            phishing_domains: HashSet::new(),
            filter_lists: Vec::new(),
            custom_rules: Vec::new(),
            stats: BlockingStats::default(),
        }
    }

    // -----------------------------------------------------------------------
    // Filter list loading
    // -----------------------------------------------------------------------

    /// Loads rules from an EasyList-format filter list string.
    ///
    /// Returns the number of rules loaded.
    pub fn load_easylist(&mut self, content: &str) -> Result<usize> {
        let mut count = 0;
        let mut hidden_count = 0;

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('!') || line.starts_with('[') {
                continue;
            }

            match self.parse_easylist_rule(line) {
                Ok(Some(rule)) => {
                    if rule.rule_type == RuleType::ElementHide {
                        let domain = rule.domain.as_deref().unwrap_or("*");
                        let selector = rule.selector.as_deref().unwrap_or("");
                        self.element_hiding_rules
                            .entry(domain.to_string())
                            .or_default()
                            .push(selector.to_string());
                        hidden_count += 1;
                    } else {
                        self.rules.push(rule);
                        count += 1;
                    }
                }
                Ok(None) => {
                    // Rule was skipped (e.g., unsupported type)
                }
                Err(e) => {
                    warn!("Failed to parse rule: {} - {}", line, e);
                }
            }
        }

        info!(
            "Loaded {} blocking rules and {} hiding rules from filter list",
            count, hidden_count
        );
        Ok(count)
    }

    /// Parses a single EasyList-format rule.
    fn parse_easylist_rule(&self, line: &str) -> Result<Option<FilterRule>> {
        let original = line.to_string();

        // Exception rules (@@)
        if line.starts_with("@@") {
            let pattern = &line[2..];
            let (domain, url_pattern) = self.extract_domain_and_pattern(pattern);
            return Ok(Some(FilterRule {
                raw: original,
                rule_type: RuleType::Allow,
                domain,
                pattern: url_pattern.clone(),
                regex: Self::pattern_to_regex(&url_pattern)?,
                enabled: true,
                selector: None,
            }));
        }

        // Element hiding rules (##)
        if line.contains("##") && !line.starts_with('/') {
            let parts: Vec<&str> = line.splitn(2, "##").collect();
            let domain = if parts[0].is_empty() {
                None
            } else {
                Some(parts[0].to_string())
            };
            let selector = parts.get(1).map(|s| s.to_string());
            return Ok(Some(FilterRule {
                raw: original,
                rule_type: RuleType::ElementHide,
                domain,
                pattern: String::new(),
                regex: Regex::new(".*")?,
                enabled: true,
                selector,
            }));
        }

        // Standard blocking rules
        let (domain, pattern) = self.extract_domain_and_pattern(line);

        // Determine rule type based on pattern content
        let rule_type = if pattern.contains("$third-party") {
            RuleType::ThirdParty
        } else if pattern.contains("$script") {
            RuleType::Script
        } else if pattern.contains("$image") {
            RuleType::Image
        } else {
            RuleType::Block
        };

        let cleaned_pattern = pattern
            .replace("$third-party", "")
            .replace("$script", "")
            .replace("$image", "")
            .replace("$xmlhttprequest", "")
            .replace("$subdocument", "")
            .trim()
            .to_string();

        Ok(Some(FilterRule {
            raw: original,
            rule_type,
            domain,
            pattern: cleaned_pattern.clone(),
            regex: Self::pattern_to_regex(&cleaned_pattern)?,
            enabled: true,
            selector: None,
        }))
    }

    /// Extracts the domain prefix and URL pattern from a rule.
    fn extract_domain_and_pattern(&self, rule: &str) -> (Option<String>, String) {
        // Check for domain-specific rules: ||domain.com^pattern
        if rule.starts_with("||") {
            let rest = &rule[2..];
            if let Some(sep_pos) = rest.find('^') {
                let domain = &rest[..sep_pos];
                let pattern = format!("||{}^", domain);
                return (Some(domain.to_string()), pattern);
            }
            return (None, rule.to_string());
        }

        // Check for domain= option
        if let Some(domain_start) = rule.find("domain=") {
            let after = &rule[domain_start + 7..];
            let domain_end = after
                .find(|c: char| c == ',' || c == '$' || c.is_whitespace())
                .unwrap_or(after.len());
            let domain = &after[..domain_end];
            let pattern = rule.replace(&format!("domain={}", domain), "");
            return (Some(domain.to_string()), pattern);
        }

        (None, rule.to_string())
    }

    /// Converts a filter pattern to a regex.
    fn pattern_to_regex(pattern: &str) -> std::result::Result<Regex, regex::Error> {
        // Escape regex special characters
        let escaped = regex::escape(pattern);

        // Convert filter syntax to regex
        let re_str = escaped
            .replace(r"\|\|", r"^(?:https?://)?(?:[^/]*\.)?") // ||domain
            .replace(r"\|", r"") // boundary anchors
            .replace(r"\*", r".*") // wildcards
            .replace(r"\^", r"[\/:;&?#=@%]?"); // separators

        let full_re = format!("(?i){}", re_str);
        Regex::new(&full_re)
    }

    // -----------------------------------------------------------------------
    // Request blocking
    // -----------------------------------------------------------------------

    /// Determines whether a URL should be blocked.
    ///
    /// # Arguments
    ///
    /// * `url` - The full URL to check.
    /// * `source_domain` - The domain of the page making the request.
    /// * `is_third_party` - Whether this is a third-party request.
    ///
    /// Returns `true` if the request should be blocked.
    pub fn should_block(&mut self, url: &str, source_domain: &str, is_third_party: bool) -> bool {
        if !self.config.enabled {
            return false;
        }

        // Check domain whitelist
        if self.config.whitelisted_domains.contains(source_domain) {
            self.stats.total_allowed += 1;
            return false;
        }

        // Check phishing domains first
        if self.config.anti_phishing {
            if let Some(domain) = self.extract_domain_from_url(url) {
                if self.phishing_domains.contains(&domain.to_lowercase()) {
                    debug!("Blocked phishing domain: {}", domain);
                    self.stats.phishing_blocked += 1;
                    self.stats.total_blocked += 1;
                    return true;
                }
            }
        }

        // Check allow rules first (whitelist takes precedence)
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }
            if rule.rule_type == RuleType::Allow && rule.regex.is_match(url) {
                self.stats.total_allowed += 1;
                return false;
            }
        }

        // Check block rules
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            // Skip allow rules (already checked)
            if rule.rule_type == RuleType::Allow
                || rule.rule_type == RuleType::ElementHide
                || rule.rule_type == RuleType::ElementStyle
            {
                continue;
            }

            // Third-party rule check
            if rule.rule_type == RuleType::ThirdParty && !is_third_party {
                continue;
            }

            if rule.regex.is_match(url) {
                debug!("Blocked by rule: {} -> {}", rule.raw, url);
                self.stats.total_blocked += 1;
                return true;
            }
        }

        // Check custom rules
        for rule in &self.custom_rules {
            if !rule.enabled {
                continue;
            }
            if let Ok(re) = Regex::new(&regex::escape(&rule.rule)) {
                if re.is_match(url) {
                    debug!("Blocked by custom rule: {} -> {}", rule.rule, url);
                    self.stats.total_blocked += 1;
                    return true;
                }
            }
        }

        self.stats.total_allowed += 1;
        false
    }

    /// Returns CSS selectors for element hiding on the given domain.
    pub fn get_element_hiding_selectors(&self, domain: &str) -> Vec<String> {
        if !self.config.enabled || !self.config.cosmetic_filtering {
            return Vec::new();
        }

        let mut selectors = Vec::new();

        // Global rules (domain = "*")
        if let Some(global) = self.element_hiding_rules.get("*") {
            selectors.extend(global.clone());
        }

        // Domain-specific rules
        if let Some(domain_rules) = self.element_hiding_rules.get(domain) {
            selectors.extend(domain_rules.clone());
        }

        selectors
    }

    /// Returns a CSS string for hiding elements on the given domain.
    pub fn get_element_hiding_css(&self, domain: &str) -> String {
        let selectors = self.get_element_hiding_selectors(domain);
        if selectors.is_empty() {
            return String::new();
        }
        selectors
            .iter()
            .map(|s| format!("{} {{ display: none !important; }}", s))
            .collect::<Vec<_>>()
            .join("\n")
    }

    // -----------------------------------------------------------------------
    // Anti-phishing
    // -----------------------------------------------------------------------

    /// Loads a list of phishing domains (one per line).
    pub fn load_phishing_list(&mut self, content: &str) -> usize {
        let mut count = 0;
        for line in content.lines() {
            let line = line.trim().to_lowercase();
            if !line.is_empty() && !line.starts_with('#') {
                self.phishing_domains.insert(line);
                count += 1;
            }
        }
        info!("Loaded {} phishing domains", count);
        count
    }

    /// Checks if a URL is a known phishing/malware site.
    pub fn check_phishing(&self, url: &str) -> bool {
        if let Some(domain) = self.extract_domain_from_url(url) {
            self.phishing_domains.contains(&domain.to_lowercase())
        } else {
            false
        }
    }

    // -----------------------------------------------------------------------
    // Filter list management
    // -----------------------------------------------------------------------

    /// Adds a filter list source.
    pub fn add_filter_list(&mut self, list: FilterList) {
        info!("Added filter list: {}", list.name);
        self.filter_lists.push(list);
    }

    /// Removes a filter list by ID.
    pub fn remove_filter_list(&mut self, id: &str) {
        self.filter_lists.retain(|l| l.id != id);
        info!("Removed filter list: {}", id);
    }

    /// Returns all installed filter lists.
    pub fn filter_lists(&self) -> &[FilterList] {
        &self.filter_lists
    }

    /// Enables or disables a filter list.
    pub fn set_filter_list_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(list) = self.filter_lists.iter_mut().find(|l| l.id == id) {
            list.enabled = enabled;
            info!(
                "Filter list {} {}",
                list.name,
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    /// Reloads all rules from filter lists. Called after updating lists.
    pub fn reload(&mut self) -> Result<()> {
        // Clear existing rules
        self.rules.clear();
        self.element_hiding_rules.clear();
        info!("Reloading all filter rules");
        // Note: In production, this would re-fetch and parse each filter list.
        // For now, the rules array is preserved; this is a reset point.
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Custom rules
    // -----------------------------------------------------------------------

    /// Adds a custom blocking rule.
    pub fn add_custom_rule(&mut self, rule: &str, description: &str) -> Result<CustomRule> {
        let id = uuid::Uuid::new_v4().to_string();
        let custom = CustomRule {
            id: id.clone(),
            rule: rule.to_string(),
            enabled: true,
            description: description.to_string(),
        };
        info!("Added custom rule: {} ({})", rule, description);
        self.custom_rules.push(custom.clone());
        Ok(custom)
    }

    /// Removes a custom rule by ID.
    pub fn remove_custom_rule(&mut self, id: &str) -> Result<()> {
        let before = self.custom_rules.len();
        self.custom_rules.retain(|r| r.id != id);
        if self.custom_rules.len() == before {
            return Err(AdBlockerError::Other(format!(
                "Custom rule not found: {}",
                id
            )));
        }
        info!("Removed custom rule: {}", id);
        Ok(())
    }

    /// Returns all custom rules.
    pub fn custom_rules(&self) -> &[CustomRule] {
        &self.custom_rules
    }

    /// Enables or disables a custom rule.
    pub fn set_custom_rule_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(rule) = self.custom_rules.iter_mut().find(|r| r.id == id) {
            rule.enabled = enabled;
            info!(
                "Custom rule {} {}",
                id,
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    // -----------------------------------------------------------------------
    // Whitelist management
    // -----------------------------------------------------------------------

    /// Adds a domain to the whitelist.
    pub fn whitelist_domain(&mut self, domain: &str) {
        self.config
            .whitelisted_domains
            .insert(domain.to_lowercase());
        info!("Whitelisted domain: {}", domain);
    }

    /// Removes a domain from the whitelist.
    pub fn remove_whitelist(&mut self, domain: &str) {
        self.config.whitelisted_domains.remove(&domain.to_lowercase());
        info!("Removed domain from whitelist: {}", domain);
    }

    /// Returns all whitelisted domains.
    pub fn whitelisted_domains(&self) -> &HashSet<String> {
        &self.config.whitelisted_domains
    }

    /// Checks if a domain is whitelisted.
    pub fn is_whitelisted(&self, domain: &str) -> bool {
        self.config
            .whitelisted_domains
            .contains(&domain.to_lowercase())
    }

    // -----------------------------------------------------------------------
    // Configuration
    // -----------------------------------------------------------------------

    /// Enables or disables the ad blocker entirely.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
        info!(
            "Ad blocker {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Returns whether the ad blocker is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &AdBlockerConfig {
        &self.config
    }

    // -----------------------------------------------------------------------
    // Statistics
    // -----------------------------------------------------------------------

    /// Returns the blocking statistics.
    pub fn stats(&self) -> &BlockingStats {
        &self.stats
    }

    /// Resets the blocking statistics.
    pub fn reset_stats(&mut self) {
        self.stats = BlockingStats::default();
        info!("Ad blocker stats reset");
    }

    /// Returns the total number of loaded rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Returns the number of element hiding rules.
    pub fn element_hiding_rule_count(&self) -> usize {
        self.element_hiding_rules.values().map(|v| v.len()).sum()
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Extracts the domain from a URL.
    fn extract_domain_from_url(&self, url: &str) -> Option<String> {
        let without_protocol = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("ftp://");
        let domain = without_protocol
            .split('/')
            .next()?
            .split(':')
            .next()?;
        Some(domain.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_easylist_rules() {
        let mut blocker = AdBlocker::new(AdBlockerConfig::default());
        let rules = r#"
! Title: Test List
||example.com/ad.js^
@@||example.com/allow.js^
example.com##.ad-banner
||tracker.com^$third-party
"#;
        let count = blocker.load_easylist(rules).unwrap();
        assert!(count > 0);
    }

    #[test]
    fn test_should_block() {
        let mut blocker = AdBlocker::new(AdBlockerConfig::default());
        blocker
            .load_easylist("||doubleclick.net^\n||googleadservices.com^\n")
            .unwrap();

        assert!(blocker.should_block(
            "https://doubleclick.net/ad.js",
            "example.com",
            true
        ));
        assert!(!blocker.should_block(
            "https://example.com/script.js",
            "example.com",
            false
        ));
    }

    #[test]
    fn test_allow_rule_takes_precedence() {
        let mut blocker = AdBlocker::new(AdBlockerConfig::default());
        blocker
            .load_easylist("||blocked.com^\n@@||blocked.com/allow.js^\n")
            .unwrap();

        assert!(!blocker.should_block(
            "https://blocked.com/allow.js",
            "example.com",
            true
        ));
        assert!(blocker.should_block(
            "https://blocked.com/ad.js",
            "example.com",
            true
        ));
    }

    #[test]
    fn test_whitelist_domain() {
        let mut blocker = AdBlocker::new(AdBlockerConfig::default());
        blocker
            .load_easylist("||doubleclick.net^\n")
            .unwrap();
        blocker.whitelist_domain("example.com");

        assert!(!blocker.should_block(
            "https://doubleclick.net/ad.js",
            "example.com",
            true
        ));
    }

    #[test]
    fn test_element_hiding() {
        let mut blocker = AdBlocker::new(AdBlockerConfig::default());
        blocker
            .load_easylist("example.com##.ad-banner\nexample.com##.popup\n")
            .unwrap();

        let selectors = blocker.get_element_hiding_selectors("example.com");
        assert_eq!(selectors.len(), 2);
        assert!(selectors.contains(&".ad-banner".to_string()));
    }

    #[test]
    fn test_phishing_check() {
        let mut blocker = AdBlocker::new(AdBlockerConfig::default());
        blocker.load_phishing_list("evil.com\nphishing.org\n");

        assert!(blocker.check_phishing("https://evil.com/login"));
        assert!(!blocker.check_phishing("https://safe.com/page"));
    }

    #[test]
    fn test_custom_rules() {
        let mut blocker = AdBlocker::new(AdBlockerConfig::default());
        blocker
            .add_custom_rule("annoying-widget.com", "Block annoying widget")
            .unwrap();

        assert_eq!(blocker.custom_rules().len(), 1);
        blocker.remove_custom_rule(&blocker.custom_rules()[0].id).unwrap();
        assert_eq!(blocker.custom_rules().len(), 0);
    }

    #[test]
    fn test_pattern_to_regex() {
        let re = AdBlocker::pattern_to_regex("||example.com/ad.js^").unwrap();
        assert!(re.is_match("https://example.com/ad.js?param=1"));
        assert!(re.is_match("http://sub.example.com/ad.js"));
        assert!(!re.is_match("https://example.com/legit.js"));
    }
}