//! Nova Browser Android - JNI bridge library
//!
//! This crate provides the native interface between Kotlin/Java Android code
//! and the Rust core libraries (nova-core, nova-features).
//! It exports JNI functions that are called from the Android app.

mod bridge;

pub use bridge::*;