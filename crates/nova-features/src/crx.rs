//! # Nova CRX Parser Module
//!
//! Parses Chrome Extension (CRX) packages in both v2 and v3 formats.
//!
//! ## CRX Format Overview
//!
//! - **CRX v2**: magic "Cr24" (4 bytes) + version (4 bytes LE) +
//!   pubkey_len (4 bytes LE) + sig_len (4 bytes LE) + public_key +
//!   signature + ZIP archive.
//!
//! - **CRX v3**: magic "Cr24" (4 bytes) + version (4 bytes LE) +
//!   header_size (4 bytes LE) + protobuf header (header_size bytes) +
//!   ZIP archive.
//!
//! ## Usage
//!
//! ```no_run
//! use std::path::Path;
//! use nova_features::crx::install_crx_from_file;
//!
//! let manifest = install_crx_from_file(
//!     Path::new("/path/to/extension.crx"),
//!     Path::new("/tmp/extracted"),
//! ).unwrap();
//! println!("Installed: {} v{}", manifest.name, manifest.version);
//! ```

use crate::extensions::ExtensionManifest;
use log::{debug, error, info};
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use thiserror::Error;
use zip::read::ZipArchive;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur during CRX parsing and installation.
#[derive(Error, Debug)]
pub enum CrxError {
    /// The file does not contain valid CRX data.
    #[error("invalid CRX magic number: expected 'Cr24', found {0:?}")]
    InvalidMagic([u8; 4]),

    /// The CRX version is not supported.
    #[error("unsupported CRX version: {0} (expected 2 or 3)")]
    UnsupportedVersion(u32),

    /// The CRX header is malformed or truncated.
    #[error("malformed CRX header: {0}")]
    MalformedHeader(String),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// The ZIP archive within the CRX is corrupt or unreadable.
    #[error("ZIP error: {0}")]
    ZipError(#[from] zip::result::ZipError),

    /// The manifest.json is missing from the archive.
    #[error("manifest.json not found in CRX archive")]
    MissingManifest,

    /// The manifest.json is invalid JSON.
    #[error("invalid manifest JSON: {0}")]
    InvalidManifestJson(String),

    /// The manifest is missing required fields.
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
}

/// Convenience type alias for CRX operations.
pub type Result<T> = std::result::Result<T, CrxError>;

// ---------------------------------------------------------------------------
// CRX Header types
// ---------------------------------------------------------------------------

/// The magic bytes that identify a valid CRX file.
const CRX_MAGIC: &[u8; 4] = b"Cr24";

/// CRX v2 header layout (16 bytes):
///
/// | Offset | Size | Field       |
/// |--------|------|-------------|
/// | 0      | 4    | magic       |
/// | 4      | 4    | version     |
/// | 8      | 4    | pubkey_len  |
/// | 12     | 4    | sig_len     |
const CRX_V2_HEADER_SIZE: usize = 16;

/// CRX v3 header layout (12 bytes):
///
/// | Offset | Size | Field       |
/// |--------|------|-------------|
/// | 0      | 4    | magic       |
/// | 4      | 4    | version     |
/// | 8      | 4    | header_size |
const CRX_V3_HEADER_SIZE: usize = 12;

/// Parsed CRX v2 header.
#[derive(Debug, Clone)]
struct CrxV2Header {
    pubkey_len: u32,
    sig_len: u32,
}

/// Parsed CRX v3 header.
#[derive(Debug, Clone)]
struct CrxV3Header {
    header_size: u32,
}

/// The result of parsing a CRX header, containing the ZIP data offset.
#[derive(Debug, Clone)]
enum CrxHeader {
    V2(CrxV2Header),
    V3(CrxV3Header),
}

impl CrxHeader {
    /// Returns the byte offset where the ZIP archive begins.
    fn zip_offset(&self) -> usize {
        match self {
            // After v2 header (16) + pubkey + signature
            CrxHeader::V2(h) => CRX_V2_HEADER_SIZE + h.pubkey_len as usize + h.sig_len as usize,
            // After v3 header (12) + protobuf header
            CrxHeader::V3(h) => CRX_V3_HEADER_SIZE + h.header_size as usize,
        }
    }
}

// ---------------------------------------------------------------------------
// CRX parsing helpers
// ---------------------------------------------------------------------------

/// Reads a little-endian u32 from a byte slice at the given offset.
fn read_u32_le(data: &[u8], offset: usize) -> Result<u32> {
    if offset + 4 > data.len() {
        return Err(CrxError::MalformedHeader(format!(
            "not enough data at offset {} to read u32 (need 4 bytes, have {})",
            offset,
            data.len().saturating_sub(offset)
        )));
    }
    let bytes: [u8; 4] = data[offset..offset + 4]
        .try_into()
        .expect("slice length already checked");
    Ok(u32::from_le_bytes(bytes))
}

/// Reads and validates the 4-byte magic number at the start of the CRX data.
fn read_magic(data: &[u8]) -> Result<[u8; 4]> {
    if data.len() < 4 {
        return Err(CrxError::MalformedHeader(
            "file too small to contain CRX magic number".to_string(),
        ));
    }
    let magic: [u8; 4] = data[..4].try_into().expect("slice length checked");
    if &magic != CRX_MAGIC {
        return Err(CrxError::InvalidMagic(magic));
    }
    Ok(magic)
}

/// Reads and validates the CRX version number.
fn read_version(data: &[u8]) -> Result<u32> {
    let version = read_u32_le(data, 4)?;
    if version != 2 && version != 3 {
        return Err(CrxError::UnsupportedVersion(version));
    }
    Ok(version)
}

/// Checks that the data slice has at least the expected number of bytes.
fn ensure_length(data: &[u8], expected: usize, context: &str) -> Result<()> {
    if data.len() < expected {
        return Err(CrxError::MalformedHeader(format!(
            "{}: expected at least {} bytes, got {}",
            context,
            expected,
            data.len()
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// CRX header parsing
// ---------------------------------------------------------------------------

/// Parses the CRX header from raw bytes.
///
/// Returns the parsed header along with the ZIP data offset.
fn parse_crx_header(data: &[u8]) -> Result<CrxHeader> {
    let _magic = read_magic(data)?;
    let version = read_version(data)?;

    match version {
        2 => {
            ensure_length(data, CRX_V2_HEADER_SIZE, "CRX v2 header")?;
            let pubkey_len = read_u32_le(data, 8)?;
            let sig_len = read_u32_le(data, 12)?;
            let total = CRX_V2_HEADER_SIZE + pubkey_len as usize + sig_len as usize;
            ensure_length(data, total, "CRX v2 full header (pubkey + signature)")?;
            info!(
                "Parsed CRX v2 header: pubkey_len={}, sig_len={}",
                pubkey_len, sig_len
            );
            Ok(CrxHeader::V2(CrxV2Header {
                pubkey_len,
                sig_len,
            }))
        }
        3 => {
            ensure_length(data, CRX_V3_HEADER_SIZE, "CRX v3 header")?;
            let header_size = read_u32_le(data, 8)?;
            let total = CRX_V3_HEADER_SIZE + header_size as usize;
            ensure_length(data, total, "CRX v3 full header")?;
            info!("Parsed CRX v3 header: header_size={}", header_size);
            Ok(CrxHeader::V3(CrxV3Header { header_size }))
        }
        _ => unreachable!("version already validated as 2 or 3"),
    }
}

// ---------------------------------------------------------------------------
// ZIP extraction
// ---------------------------------------------------------------------------

/// A fully parsed CRX package containing the manifest and all extracted files.
#[derive(Debug, Clone)]
pub struct CrxPackage {
    /// The parsed extension manifest.
    pub manifest: ExtensionManifest,
    /// All files extracted from the ZIP archive, keyed by relative path.
    pub files: HashMap<String, Vec<u8>>,
}

/// Extracts the ZIP archive from the CRX data starting at `zip_offset`.
///
/// Returns a `HashMap` of file paths to their raw byte contents.
/// The `manifest.json` entry is validated and returned separately as part of
/// the `CrxPackage`.
fn extract_zip(data: &[u8], zip_offset: usize) -> Result<CrxPackage> {
    let zip_data = &data[zip_offset..];
    let cursor = Cursor::new(zip_data);
    let mut archive = ZipArchive::new(cursor)?;

    let mut files: HashMap<String, Vec<u8>> = HashMap::with_capacity(archive.len());
    let mut manifest_json: Option<String> = None;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        debug!("Extracting CRX entry: {}", name);

        // Skip directories
        if entry.is_dir() {
            continue;
        }

        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf)?;

        if name == "manifest.json" {
            manifest_json = Some(
                String::from_utf8(buf.clone())
                    .map_err(|e| CrxError::InvalidManifestJson(e.to_string()))?,
            );
        }

        files.insert(name, buf);
    }

    let manifest_json = manifest_json.ok_or(CrxError::MissingManifest)?;

    let manifest: ExtensionManifest = serde_json::from_str(&manifest_json)
        .map_err(|e| CrxError::InvalidManifestJson(e.to_string()))?;

    // Basic manifest validation
    validate_manifest(&manifest)?;

    Ok(CrxPackage { manifest, files })
}

/// Validates that the extension manifest has all required fields.
fn validate_manifest(manifest: &ExtensionManifest) -> Result<()> {
    if manifest.id.is_empty() {
        return Err(CrxError::InvalidManifest(
            "manifest.id is empty".to_string(),
        ));
    }
    if manifest.name.is_empty() {
        return Err(CrxError::InvalidManifest(
            "manifest.name is empty".to_string(),
        ));
    }
    if manifest.version.is_empty() {
        return Err(CrxError::InvalidManifest(
            "manifest.version is empty".to_string(),
        ));
    }
    debug!(
        "Validated manifest: {} v{} (id={})",
        manifest.name, manifest.version, manifest.id
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parses a CRX package from raw bytes.
///
/// This function:
/// 1. Parses and validates the CRX header (v2 or v3).
/// 2. Extracts the embedded ZIP archive.
/// 3. Locates and parses `manifest.json`.
/// 4. Validates the extension manifest.
///
/// Returns a [`CrxPackage`] containing the manifest and all extracted files.
///
/// # Errors
///
/// Returns [`CrxError`] if the data is not a valid CRX package, the ZIP
/// archive is corrupt, or the manifest is missing or invalid.
pub fn install_crx(data: &[u8]) -> Result<CrxPackage> {
    if data.is_empty() {
        return Err(CrxError::MalformedHeader("CRX data is empty".to_string()));
    }

    let header = parse_crx_header(data)?;
    let zip_offset = header.zip_offset();

    ensure_length(
        data,
        zip_offset + 1,
        "CRX data (ZIP archive must follow header)",
    )?;

    info!(
        "Parsing CRX: version={}, zip_offset={}, total_size={}",
        match &header {
            CrxHeader::V2(_) => 2,
            CrxHeader::V3(_) => 3,
        },
        zip_offset,
        data.len()
    );

    extract_zip(data, zip_offset)
}

/// Installs a CRX extension from a file path, extracting its contents to a
/// destination directory.
///
/// This is a convenience wrapper around [`install_crx`] that reads the file
/// from disk and writes the extracted files to `dest_dir`.
///
/// # Arguments
///
/// * `path` - Path to the `.crx` file.
/// * `dest_dir` - Directory where the extracted extension files will be written.
///
/// # Returns
///
/// The parsed [`ExtensionManifest`] on success.
///
/// # Errors
///
/// Returns [`CrxError`] on any parsing or I/O failure.
pub fn install_crx_from_file(path: &Path, dest_dir: &Path) -> Result<ExtensionManifest> {
    info!("Installing CRX from file: {:?}", path);

    let data = std::fs::read(path).map_err(|e| {
        error!("Failed to read CRX file {:?}: {}", path, e);
        CrxError::IoError(e)
    })?;

    let package = install_crx(&data)?;

    // Write extracted files to the destination directory
    std::fs::create_dir_all(dest_dir).map_err(|e| {
        error!("Failed to create destination directory {:?}: {}", dest_dir, e);
        CrxError::IoError(e)
    })?;

    for (file_path, content) in &package.files {
        let full_path = dest_dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                error!("Failed to create directory {:?}: {}", parent, e);
                CrxError::IoError(e)
            })?;
        }
        let mut f = std::fs::File::create(&full_path).map_err(|e| {
            error!("Failed to create file {:?}: {}", full_path, e);
            CrxError::IoError(e)
        })?;
        f.write_all(content).map_err(|e| {
            error!("Failed to write file {:?}: {}", full_path, e);
            CrxError::IoError(e)
        })?;
        debug!("Wrote extracted file: {:?}", full_path);
    }

    info!(
        "Successfully installed CRX: {} v{}",
        package.manifest.name, package.manifest.version
    );

    Ok(package.manifest)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build a minimal CRX v2 byte buffer for testing.
    fn make_crx_v2(manifest_json: &str, extra_files: &[(&str, &[u8])]) -> Vec<u8> {
        let pubkey = b"fake-public-key-data-1234";
        let signature = b"fake-signature-data-5678";

        // Build a zip in memory
        let mut zip_buf = Vec::new();
        {
            let mut zip_writer =
                zip::write::ZipWriter::new(Cursor::new(&mut zip_buf));
            let options =
                zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

            zip_writer
                .start_file("manifest.json", options)
                .unwrap();
            zip_writer
                .write_all(manifest_json.as_bytes())
                .unwrap();

            for (name, data) in extra_files {
                zip_writer.start_file(*name, options).unwrap();
                zip_writer.write_all(data).unwrap();
            }

            zip_writer.finish().unwrap();
        }

        let mut buf = Vec::new();
        // Magic "Cr24"
        buf.extend_from_slice(b"Cr24");
        // Version 2
        buf.extend_from_slice(&2u32.to_le_bytes());
        // pubkey_len
        buf.extend_from_slice(&(pubkey.len() as u32).to_le_bytes());
        // sig_len
        buf.extend_from_slice(&(signature.len() as u32).to_le_bytes());
        // Public key
        buf.extend_from_slice(pubkey);
        // Signature
        buf.extend_from_slice(signature);
        // ZIP data
        buf.extend_from_slice(&zip_buf);

        buf
    }

    fn make_test_manifest_json() -> String {
        r#"{
            "id": "test-ext-001",
            "name": "Test CRX Extension",
            "description": "A test extension for CRX parsing",
            "version": "1.0.0",
            "author": "Test Author",
            "permissions": ["Storage", "Tabs"],
            "background_script": "background.js",
            "default_popup": "popup.html"
        }"#
        .to_string()
    }

    #[test]
    fn test_parse_crx_v2_header_valid() {
        let data = make_crx_v2(&make_test_manifest_json(), &[]);
        let header = parse_crx_header(&data).unwrap();
        match header {
            CrxHeader::V2(h) => {
                assert_eq!(h.pubkey_len, 24);
                assert_eq!(h.sig_len, 24);
            }
            _ => panic!("expected V2 header"),
        }
    }

    #[test]
    fn test_parse_crx_v3_header() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"Cr24");
        buf.extend_from_slice(&3u32.to_le_bytes());
        let header_proto: &[u8] = &[0x0a, 0x14, 0x12, 0x12]; // minimal protobuf header
        buf.extend_from_slice(&(header_proto.len() as u32).to_le_bytes());
        buf.extend_from_slice(header_proto);
        // Add minimal ZIP data
        buf.push(0x50); // at least 1 byte after header

        let header = parse_crx_header(&buf).unwrap();
        match header {
            CrxHeader::V3(h) => {
                assert_eq!(h.header_size, 4);
            }
            _ => panic!("expected V3 header"),
        }
    }

    #[test]
    fn test_crx_invalid_magic() {
        let data = b"Xr24bad";
        let err = parse_crx_header(data).unwrap_err();
        assert!(matches!(err, CrxError::InvalidMagic(_)));
    }

    #[test]
    fn test_crx_unsupported_version() {
        let mut data = Vec::from(*b"Cr24");
        data.extend_from_slice(&99u32.to_le_bytes());
        let err = parse_crx_header(&data).unwrap_err();
        assert!(matches!(err, CrxError::UnsupportedVersion(99)));
    }

    #[test]
    fn test_crx_truncated_header() {
        let data = b"Cr24";
        let err = parse_crx_header(data).unwrap_err();
        assert!(matches!(err, CrxError::MalformedHeader(_)));
    }

    #[test]
    fn test_install_crx_v2_full() {
        let manifest_json = make_test_manifest_json();
        let bg_script = b"console.log('background');";
        let data = make_crx_v2(&manifest_json, &[("background.js", bg_script)]);

        let package = install_crx(&data).unwrap();
        assert_eq!(package.manifest.id, "test-ext-001");
        assert_eq!(package.manifest.name, "Test CRX Extension");
        assert_eq!(package.manifest.version, "1.0.0");
        assert!(package.files.contains_key("manifest.json"));
        assert!(package.files.contains_key("background.js"));
    }

    #[test]
    fn test_install_crx_missing_manifest() {
        let pubkey = b"fake-public-key";
        let signature = b"fake-signature";

        let mut zip_buf = Vec::new();
        {
            let mut zip_writer =
                zip::write::ZipWriter::new(Cursor::new(&mut zip_buf));
            let options =
                zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip_writer
                .start_file("not-manifest.txt", options)
                .unwrap();
            zip_writer.write_all(b"not a manifest").unwrap();
            zip_writer.finish().unwrap();
        }

        let mut buf = Vec::new();
        buf.extend_from_slice(b"Cr24");
        buf.extend_from_slice(&2u32.to_le_bytes());
        buf.extend_from_slice(&(pubkey.len() as u32).to_le_bytes());
        buf.extend_from_slice(&(signature.len() as u32).to_le_bytes());
        buf.extend_from_slice(pubkey);
        buf.extend_from_slice(signature);
        buf.extend_from_slice(&zip_buf);

        let err = install_crx(&buf).unwrap_err();
        assert!(matches!(err, CrxError::MissingManifest));
    }

    #[test]
    fn test_install_crx_empty_data() {
        let err = install_crx(&[]).unwrap_err();
        assert!(matches!(err, CrxError::MalformedHeader(_)));
    }

    #[test]
    fn test_validate_manifest_empty_id() {
        let manifest = ExtensionManifest {
            id: String::new(),
            name: "Test".to_string(),
            description: "desc".to_string(),
            version: "1.0".to_string(),
            author: None,
            homepage_url: None,
            minimum_browser_version: None,
            permissions: vec![],
            host_permissions: vec![],
            dependencies: vec![],
            background_script: None,
            content_scripts: vec![],
            icons: HashMap::new(),
            default_popup: None,
            options_page: None,
        };
        let err = validate_manifest(&manifest).unwrap_err();
        assert!(matches!(err, CrxError::InvalidManifest(_)));
        assert!(err.to_string().contains("manifest.id"));
    }

    #[test]
    fn test_zip_offset_calculations() {
        let v2 = CrxHeader::V2(CrxV2Header {
            pubkey_len: 100,
            sig_len: 200,
        });
        assert_eq!(v2.zip_offset(), 16 + 100 + 200);

        let v3 = CrxHeader::V3(CrxV3Header {
            header_size: 150,
        });
        assert_eq!(v3.zip_offset(), 12 + 150);
    }
}