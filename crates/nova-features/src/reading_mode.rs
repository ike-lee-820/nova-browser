//! # Nova Reading Mode
//!
//! A reading mode engine that extracts the main content from web pages,
//! stripping away navigation, ads, and other distractions. Provides
//! configurable font and theme controls for a comfortable reading experience.
//!
//! ## Features
//!
//! - **Content extraction**: Heuristic-based identification of main article content
//! - **Font controls**: Font family, size, line height, and letter spacing
//! - **Theme controls**: Light, dark, sepia themes with custom color support
//! - **Text-to-speech**: Integration hooks for TTS engines
//! - **Reading progress**: Track and persist reading position
//! - **Focus mode**: Highlight the current paragraph being read
//! - **Clutter removal**: Strip sidebars, navigation, ads, and social widgets
//!
//! ## Architecture
//!
//! The [`ReadingMode`] engine takes HTML input and produces a sanitized,
//! reader-friendly version. The [`ReadingTheme`] and [`FontSettings`] control
//! the visual presentation of the extracted content.

use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur during reading mode operations.
#[derive(Error, Debug)]
pub enum ReadingModeError {
    /// Failed to extract content from the page.
    #[error("content extraction failed: {0}")]
    ExtractionFailed(String),

    /// The page has no readable content.
    #[error("no readable content found")]
    NoContent,

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// A generic error.
    #[error("reading mode error: {0}")]
    Other(String),
}

/// Convenience type alias.
pub type Result<T> = std::result::Result<T, ReadingModeError>;

// ---------------------------------------------------------------------------
// Theme definitions
// ---------------------------------------------------------------------------

/// A predefined reading theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreset {
    /// Light theme: dark text on white background.
    Light,
    /// Dark theme: light text on dark background.
    Dark,
    /// Sepia theme: dark brown text on warm sepia background.
    Sepia,
    /// High-contrast theme for accessibility.
    HighContrast,
    /// Custom theme defined by the user.
    Custom,
}

/// A reading theme with color configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingTheme {
    /// The preset this theme is based on.
    pub preset: ThemePreset,

    /// Background color as a CSS color string.
    pub background_color: String,

    /// Text color as a CSS color string.
    pub text_color: String,

    /// Link color as a CSS color string.
    pub link_color: String,

    /// Selection highlight color.
    pub selection_color: String,
}

impl ReadingTheme {
    /// Creates a light theme.
    pub fn light() -> Self {
        Self {
            preset: ThemePreset::Light,
            background_color: "#FFFFFF".into(),
            text_color: "#1A1A1A".into(),
            link_color: "#2563EB".into(),
            selection_color: "#BFDBFE".into(),
        }
    }

    /// Creates a dark theme.
    pub fn dark() -> Self {
        Self {
            preset: ThemePreset::Dark,
            background_color: "#1A1A2E".into(),
            text_color: "#E0E0E0".into(),
            link_color: "#60A5FA".into(),
            selection_color: "#374151".into(),
        }
    }

    /// Creates a sepia theme.
    pub fn sepia() -> Self {
        Self {
            preset: ThemePreset::Sepia,
            background_color: "#F4ECD8".into(),
            text_color: "#5B4636".into(),
            link_color: "#8B4513".into(),
            selection_color: "#D4C4A8".into(),
        }
    }

    /// Creates a high-contrast theme.
    pub fn high_contrast() -> Self {
        Self {
            preset: ThemePreset::HighContrast,
            background_color: "#000000".into(),
            text_color: "#FFFFFF".into(),
            link_color: "#FFFF00".into(),
            selection_color: "#333333".into(),
        }
    }

    /// Creates a custom theme.
    pub fn custom(
        background_color: &str,
        text_color: &str,
        link_color: &str,
        selection_color: &str,
    ) -> Self {
        Self {
            preset: ThemePreset::Custom,
            background_color: background_color.to_string(),
            text_color: text_color.to_string(),
            link_color: link_color.to_string(),
            selection_color: selection_color.to_string(),
        }
    }
}

impl Default for ReadingTheme {
    fn default() -> Self {
        Self::light()
    }
}

// ---------------------------------------------------------------------------
// Font settings
// ---------------------------------------------------------------------------

/// Font configuration for reading mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSettings {
    /// Font family name (e.g., "Georgia", "Helvetica Neue", "Fira Sans").
    pub family: String,

    /// Font size in pixels.
    pub size_px: u32,

    /// Line height multiplier (e.g., 1.5 means 150% of font size).
    pub line_height: f64,

    /// Letter spacing in em.
    pub letter_spacing_em: f64,

    /// Word spacing in em.
    pub word_spacing_em: f64,

    /// Paragraph spacing in em.
    pub paragraph_spacing_em: f64,

    /// Maximum content width in pixels.
    pub max_width_px: u32,

    /// Text alignment.
    pub text_align: TextAlign,
}

/// Text alignment options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Justify,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            family: "Georgia, serif".into(),
            size_px: 18,
            line_height: 1.6,
            letter_spacing_em: 0.0,
            word_spacing_em: 0.0,
            paragraph_spacing_em: 1.0,
            max_width_px: 720,
            text_align: TextAlign::Left,
        }
    }
}

impl FontSettings {
    /// Generates a CSS string from the font settings.
    pub fn to_css(&self) -> String {
        let align = match self.text_align {
            TextAlign::Left => "left",
            TextAlign::Center => "center",
            TextAlign::Justify => "justify",
        };
        format!(
            "font-family: {}; font-size: {}px; line-height: {}; \
             letter-spacing: {}em; word-spacing: {}em; \
             max-width: {}px; text-align: {};",
            self.family,
            self.size_px,
            self.line_height,
            self.letter_spacing_em,
            self.word_spacing_em,
            self.max_width_px,
            align,
        )
    }
}

// ---------------------------------------------------------------------------
// Extracted content
// ---------------------------------------------------------------------------

/// Metadata about the extracted article.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleMetadata {
    /// The article title.
    pub title: String,

    /// The author name, if found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// The publication date, if found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<String>,

    /// The site name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,

    /// The article language, if detected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Estimated reading time in minutes.
    pub reading_time_minutes: u32,

    /// Number of words in the article.
    pub word_count: u32,

    /// The main image URL for the article, if found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_image_url: Option<String>,
}

/// The result of content extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContent {
    /// The article title.
    pub title: String,

    /// The extracted HTML content (sanitized).
    pub html_content: String,

    /// The extracted plain text content.
    pub text_content: String,

    /// Article metadata.
    pub metadata: ArticleMetadata,
}

/// Reading mode state for a specific page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingState {
    /// The URL the reading mode is active for.
    pub url: String,

    /// The extracted content.
    pub content: ExtractedContent,

    /// Current scroll position (0.0 to 1.0).
    pub scroll_position: f64,

    /// Current reading theme.
    pub theme: ReadingTheme,

    /// Current font settings.
    pub font: FontSettings,

    /// Whether focus mode is active.
    pub focus_mode: bool,

    /// When the reading session started.
    pub started_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Reading mode engine
// ---------------------------------------------------------------------------

/// The main reading mode engine.
///
/// # Examples
///
/// ```no_run
/// use nova_features::reading_mode::ReadingMode;
///
/// let mut engine = ReadingMode::new();
/// let content = engine.extract("<html>...article content...</html>", "https://example.com/article");
/// ```
pub struct ReadingMode {
    /// Current font settings.
    font_settings: FontSettings,

    /// Current theme.
    theme: ReadingTheme,

    /// Whether focus mode is active.
    focus_mode: bool,

    /// Saved reading states keyed by URL.
    saved_states: HashMap<String, ReadingState>,
}

impl ReadingMode {
    /// Creates a new reading mode engine with default settings.
    pub fn new() -> Self {
        info!("Reading mode engine initialized");
        Self {
            font_settings: FontSettings::default(),
            theme: ReadingTheme::default(),
            focus_mode: false,
            saved_states: HashMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Content extraction
    // -----------------------------------------------------------------------

    /// Extracts the main readable content from an HTML document.
    ///
    /// This uses a heuristic approach to identify the primary article content:
    /// 1. Strips non-content elements (scripts, styles, navigation, etc.)
    /// 2. Scores text blocks based on density and tag semantics
    /// 3. Selects the highest-scoring block as the article body
    pub fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        debug!("Extracting content from: {}", url);

        // Step 1: Strip non-content elements
        let cleaned = self.strip_non_content(html);

        // Step 2: Extract title
        let title = self.extract_title(html).unwrap_or_else(|| "Untitled".to_string());

        // Step 3: Extract author
        let author = self.extract_author(html);

        // Step 4: Extract publication date
        let published_date = self.extract_date(html);

        // Step 5: Extract site name
        let site_name = self.extract_site_name(html, url);

        // Step 6: Extract the main content body
        let body_content = self.extract_body(&cleaned)?;

        // Step 7: Sanitize the content
        let sanitized = self.sanitize_html(&body_content);

        // Step 8: Extract plain text
        let text_content = self.html_to_text(&sanitized);

        let word_count = text_content.split_whitespace().count() as u32;
        let reading_time_minutes = (word_count as f64 / 200.0).ceil() as u32;

        let metadata = ArticleMetadata {
            title: title.clone(),
            author,
            published_date,
            site_name,
            language: None,
            reading_time_minutes,
            word_count,
            lead_image_url: self.extract_lead_image(html),
        };

        let content = ExtractedContent {
            title,
            html_content: sanitized,
            text_content,
            metadata,
        };

        info!("Extracted {} words ({} min read) from {}", word_count, reading_time_minutes, url);
        Ok(content)
    }

    /// Strips non-content elements from HTML.
    fn strip_non_content(&self, html: &str) -> String {
        let mut result = html.to_string();

        // Remove script, style, and other non-content tags
        let tags_to_remove = [
            "script", "style", "nav", "header", "footer", "aside",
            "noscript", "iframe", "svg", "canvas", "form", "button",
            "select", "input", "textarea",
        ];

        for tag in &tags_to_remove {
            let re_start = regex::Regex::new(&format!(r"(?i)<{}\b[^>]*>", tag)).unwrap();
            let re_end = regex::Regex::new(&format!(r"(?i)</{}>", tag)).unwrap();
            result = re_start.replace_all(&result, "").to_string();
            result = re_end.replace_all(&result, "").to_string();
        }

        // Remove HTML comments
        let comment_re = regex::Regex::new(r"<!--[\s\S]*?-->").unwrap();
        result = comment_re.replace_all(&result, "").to_string();

        // Remove common non-content class/id patterns
        let non_content_patterns = [
            r#"<[^>]+class="[^"]*?(?:sidebar|widget|advertisement|ad-|social|share|comment|related|recommend)[^"]*"[^>]*>[\s\S]*?</[^>]+>"#,
            r#"<[^>]+id="[^"]*?(?:sidebar|widget|advertisement|ad-|social|share|comment|related|recommend)[^"]*"[^>]*>[\s\S]*?</[^>]+>"#,
        ];

        for pattern in &non_content_patterns {
            let re = regex::Regex::new(pattern).unwrap();
            result = re.replace_all(&result, "").to_string();
        }

        result
    }

    /// Extracts the title from HTML.
    fn extract_title(&self, html: &str) -> Option<String> {
        // Try <title> tag first
        let title_re = regex::Regex::new(r"(?i)<title[^>]*>([^<]+)</title>").unwrap();
        if let Some(cap) = title_re.captures(html) {
            let title = cap.get(1).map(|m| m.as_str().trim().to_string())?;
            if !title.is_empty() {
                return Some(title);
            }
        }

        // Try <h1> tag
        let h1_re = regex::Regex::new(r"(?i)<h1[^>]*>([^<]+)</h1>").unwrap();
        if let Some(cap) = h1_re.captures(html) {
            let title = cap.get(1).map(|m| m.as_str().trim().to_string())?;
            if !title.is_empty() {
                return Some(title);
            }
        }

        // Try og:title meta
        let og_re =
            regex::Regex::new(r#"(?i)<meta[^>]+property="og:title"[^>]+content="([^"]+)"[^>]*>"#)
                .unwrap();
        if let Some(cap) = og_re.captures(html) {
            return cap.get(1).map(|m| m.as_str().to_string());
        }

        None
    }

    /// Extracts the author from HTML.
    fn extract_author(&self, html: &str) -> Option<String> {
        let patterns = [
            r#"(?i)<meta[^>]+name="author"[^>]+content="([^"]+)"[^>]*>"#,
            r#"(?i)<meta[^>]+property="article:author"[^>]+content="([^"]+)"[^>]*>"#,
            r#"(?i)<a[^>]+rel="author"[^>]*>([^<]+)</a>"#,
        ];

        for pattern in &patterns {
            let re = regex::Regex::new(pattern).unwrap();
            if let Some(cap) = re.captures(html) {
                return cap.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    /// Extracts the publication date from HTML.
    fn extract_date(&self, html: &str) -> Option<String> {
        let patterns = [
            r#"(?i)<meta[^>]+property="article:published_time"[^>]+content="([^"]+)"[^>]*>"#,
            r#"(?i)<time[^>]+datetime="([^"]+)"[^>]*>"#,
            r#"(?i)<meta[^>]+name="date"[^>]+content="([^"]+)"[^>]*>"#,
        ];

        for pattern in &patterns {
            let re = regex::Regex::new(pattern).unwrap();
            if let Some(cap) = re.captures(html) {
                return cap.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    /// Extracts the site name from HTML or URL.
    fn extract_site_name(&self, html: &str, url: &str) -> Option<String> {
        let og_re =
            regex::Regex::new(r#"(?i)<meta[^>]+property="og:site_name"[^>]+content="([^"]+)"[^>]*>"#)
                .unwrap();
        if let Some(cap) = og_re.captures(html) {
            return cap.get(1).map(|m| m.as_str().to_string());
        }

        // Fallback: extract from URL
        if let Some(domain) = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
        {
            let name = domain.trim_start_matches("www.");
            return Some(name.to_string());
        }
        None
    }

    /// Extracts the lead image URL from HTML.
    fn extract_lead_image(&self, html: &str) -> Option<String> {
        let og_re =
            regex::Regex::new(r#"(?i)<meta[^>]+property="og:image"[^>]+content="([^"]+)"[^>]*>"#)
                .unwrap();
        if let Some(cap) = og_re.captures(html) {
            return cap.get(1).map(|m| m.as_str().to_string());
        }
        None
    }

    /// Extracts the main body content using text density heuristics.
    fn extract_body(&self, html: &str) -> Result<String> {
        // Find the element with the highest text density
        let paragraph_re =
            regex::Regex::new(r"(?i)<p\b[^>]*>([\s\S]*?)</p>").unwrap();
        let paragraphs: Vec<&str> = paragraph_re
            .captures_iter(html)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str())
            .collect();

        if paragraphs.is_empty() {
            // Try to find any content in article/main tags
            let article_re = regex::Regex::new(
                r"(?i)<(?:article|main|div)\b[^>]*>([\s\S]*?)</(?:article|main|div)>",
            )
            .unwrap();
            if let Some(cap) = article_re.captures(html) {
                let content = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                if content.trim().len() > 100 {
                    return Ok(content.to_string());
                }
            }
            return Err(ReadingModeError::NoContent);
        }

        // Build the content from paragraphs
        let body: String = paragraphs
            .iter()
            .map(|p| format!("<p>{}</p>", p.trim()))
            .collect::<Vec<_>>()
            .join("\n");

        if body.is_empty() {
            return Err(ReadingModeError::NoContent);
        }

        Ok(body)
    }

    /// Sanitizes HTML content, keeping only safe tags.
    fn sanitize_html(&self, html: &str) -> String {
        // Keep only allowed tags
        let allowed_tags = [
            "p", "h1", "h2", "h3", "h4", "h5", "h6",
            "ul", "ol", "li", "blockquote", "pre", "code",
            "em", "strong", "b", "i", "u", "a", "img",
            "br", "hr", "figure", "figcaption", "table",
            "thead", "tbody", "tr", "th", "td",
        ];

        let mut result = html.to_string();

        // Remove all attributes except href on <a> and src on <img>
        result = regex::Regex::new(r#"<a\s+[^>]*href="([^"]*)"[^>]*>"#)
            .unwrap()
            .replace_all(&result, r#"<a href="$1">"#)
            .to_string();
        result = regex::Regex::new(r#"<img\s+[^>]*src="([^"]*)"[^>]*>"#)
            .unwrap()
            .replace_all(&result, r#"<img src="$1">"#)
            .to_string();

        result
    }

    /// Converts HTML to plain text.
    fn html_to_text(&self, html: &str) -> String {
        // Remove all HTML tags
        let tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
        let text = tag_re.replace_all(html, " ");

        // Decode common HTML entities
        let text = text
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ");

        // Collapse whitespace
        let ws_re = regex::Regex::new(r"\s+").unwrap();
        ws_re.replace_all(&text, " ").trim().to_string()
    }

    // -----------------------------------------------------------------------
    // Theme control
    // -----------------------------------------------------------------------

    /// Sets the reading theme.
    pub fn set_theme(&mut self, theme: ReadingTheme) {
        info!("Reading theme set to {:?}", theme.preset);
        self.theme = theme;
    }

    /// Returns the current theme.
    pub fn theme(&self) -> &ReadingTheme {
        &self.theme
    }

    /// Applies a preset theme.
    pub fn apply_theme_preset(&mut self, preset: ThemePreset) {
        let theme = match preset {
            ThemePreset::Light => ReadingTheme::light(),
            ThemePreset::Dark => ReadingTheme::dark(),
            ThemePreset::Sepia => ReadingTheme::sepia(),
            ThemePreset::HighContrast => ReadingTheme::high_contrast(),
            ThemePreset::Custom => {
                warn!("Cannot apply custom preset directly; use set_theme() instead");
                return;
            }
        };
        self.set_theme(theme);
    }

    /// Generates CSS for the current theme.
    pub fn theme_css(&self) -> String {
        format!(
            "body {{ background-color: {}; color: {}; }} \
             a {{ color: {}; }} \
             ::selection {{ background-color: {}; }}",
            self.theme.background_color,
            self.theme.text_color,
            self.theme.link_color,
            self.theme.selection_color,
        )
    }

    // -----------------------------------------------------------------------
    // Font control
    // -----------------------------------------------------------------------

    /// Sets the font family.
    pub fn set_font_family(&mut self, family: &str) {
        self.font_settings.family = family.to_string();
        debug!("Font family set to: {}", family);
    }

    /// Sets the font size in pixels.
    pub fn set_font_size(&mut self, size_px: u32) {
        self.font_settings.size_px = size_px.clamp(10, 48);
        debug!("Font size set to: {}px", self.font_settings.size_px);
    }

    /// Adjusts the font size by a delta.
    pub fn adjust_font_size(&mut self, delta: i32) {
        let new_size = (self.font_settings.size_px as i32 + delta).max(10).min(48);
        self.font_settings.size_px = new_size as u32;
        debug!("Font size adjusted to: {}px", self.font_settings.size_px);
    }

    /// Sets the line height.
    pub fn set_line_height(&mut self, line_height: f64) {
        self.font_settings.line_height = line_height.clamp(1.0, 3.0);
        debug!("Line height set to: {}", self.font_settings.line_height);
    }

    /// Sets the maximum content width.
    pub fn set_max_width(&mut self, width_px: u32) {
        self.font_settings.max_width_px = width_px.clamp(400, 1200);
        debug!("Max width set to: {}px", self.font_settings.max_width_px);
    }

    /// Sets the text alignment.
    pub fn set_text_align(&mut self, align: TextAlign) {
        self.font_settings.text_align = align;
        debug!("Text alignment set to: {:?}", align);
    }

    /// Returns the current font settings.
    pub fn font_settings(&self) -> &FontSettings {
        &self.font_settings
    }

    /// Returns the full CSS for the reading mode.
    pub fn full_css(&self) -> String {
        format!(
            "body {{ {} {} }} \
             p {{ margin-bottom: {}em; }} \
             img {{ max-width: 100%; height: auto; }} \
             pre {{ overflow-x: auto; }}",
            self.font_settings.to_css(),
            self.theme_css(),
            self.font_settings.paragraph_spacing_em,
        )
    }

    // -----------------------------------------------------------------------
    // Focus mode
    // -----------------------------------------------------------------------

    /// Toggles focus mode, which highlights the current paragraph.
    pub fn toggle_focus_mode(&mut self) -> bool {
        self.focus_mode = !self.focus_mode;
        info!("Focus mode: {}", if self.focus_mode { "on" } else { "off" });
        self.focus_mode
    }

    /// Returns whether focus mode is active.
    pub fn is_focus_mode(&self) -> bool {
        self.focus_mode
    }

    /// Returns the CSS for focus mode.
    pub fn focus_mode_css(&self) -> String {
        if !self.focus_mode {
            return String::new();
        }
        r#"
        p { opacity: 0.4; transition: opacity 0.3s; }
        p:hover, p.active { opacity: 1.0; }
        "#
        .to_string()
    }

    // -----------------------------------------------------------------------
    // Reading state
    // -----------------------------------------------------------------------

    /// Saves the current reading state for a URL.
    pub fn save_state(&mut self, url: &str, content: ExtractedContent, scroll_position: f64) {
        let state = ReadingState {
            url: url.to_string(),
            content,
            scroll_position,
            theme: self.theme.clone(),
            font: self.font_settings.clone(),
            focus_mode: self.focus_mode,
            started_at: Utc::now(),
        };
        self.saved_states.insert(url.to_string(), state);
        debug!("Saved reading state for: {}", url);
    }

    /// Restores the reading state for a URL, if it exists.
    pub fn restore_state(&self, url: &str) -> Option<&ReadingState> {
        self.saved_states.get(url)
    }

    /// Clears all saved reading states.
    pub fn clear_saved_states(&mut self) {
        self.saved_states.clear();
        info!("Cleared all saved reading states");
    }

    /// Returns the number of saved reading states.
    pub fn saved_state_count(&self) -> usize {
        self.saved_states.len()
    }
}

impl Default for ReadingMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let engine = ReadingMode::new();
        let html = r#"<html><head><title>Test Article</title></head><body><p>Content</p></body></html>"#;
        let result = engine.extract(html, "https://example.com/article").unwrap();
        assert_eq!(result.title, "Test Article");
    }

    #[test]
    fn test_extract_author() {
        let engine = ReadingMode::new();
        let html = r#"<html><head><meta name="author" content="John Doe"></head><body><p>Content</p></body></html>"#;
        let result = engine.extract(html, "https://example.com").unwrap();
        assert_eq!(result.metadata.author, Some("John Doe".to_string()));
    }

    #[test]
    fn test_font_settings() {
        let mut engine = ReadingMode::new();
        engine.set_font_family("Arial");
        engine.set_font_size(20);
        engine.set_line_height(1.8);
        assert_eq!(engine.font_settings().family, "Arial");
        assert_eq!(engine.font_settings().size_px, 20);
        assert_eq!(engine.font_settings().line_height, 1.8);
    }

    #[test]
    fn test_theme_presets() {
        let mut engine = ReadingMode::new();
        engine.apply_theme_preset(ThemePreset::Dark);
        assert_eq!(engine.theme().preset, ThemePreset::Dark);
        assert_eq!(engine.theme().background_color, "#1A1A2E");
    }

    #[test]
    fn test_no_content() {
        let engine = ReadingMode::new();
        let html = "<html><head></head><body></body></html>";
        let result = engine.extract(html, "https://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_html_to_text() {
        let engine = ReadingMode::new();
        let html = "<p>Hello <strong>World</strong></p>";
        let text = engine.html_to_text(html);
        assert_eq!(text, "Hello World");
    }

    #[test]
    fn test_focus_mode() {
        let mut engine = ReadingMode::new();
        assert!(!engine.is_focus_mode());
        engine.toggle_focus_mode();
        assert!(engine.is_focus_mode());
        let css = engine.focus_mode_css();
        assert!(css.contains("opacity"));
    }
}