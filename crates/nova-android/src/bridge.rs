//! JNI bridge functions exposed to the Android Kotlin/Java layer.
//!
//! These functions are called from `com.novabrowser.bridge.RustBridge`.

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jstring, JNI_TRUE, JNI_FALSE};
use std::sync::Mutex;

use nova_core::config::AppConfig;
use nova_features::FeatureManager;

static FEATURE_MANAGER: Mutex<Option<FeatureManager>> = Mutex::new(None);
static APP_CONFIG: Mutex<Option<AppConfig>> = Mutex::new(None);

/// Helper to extract a Rust String from a JString (jni 0.20 takes JString by value).
fn jstring_to_string(env: &mut JNIEnv, s: JString) -> Result<String, jni::errors::Error> {
    env.get_string(s).map(|js| js.into())
}

/// Helper to get mutable access to the feature manager.
fn with_manager<F, R>(f: F) -> R
where
    F: FnOnce(&mut FeatureManager) -> R,
{
    let mut guard = FEATURE_MANAGER.lock().expect("FeatureManager lock poisoned");
    let mgr = guard.as_mut().expect("FeatureManager not initialized");
    f(mgr)
}

/// Helper to get optional mutable access to the feature manager.
fn with_manager_opt<F, R>(default: R, f: F) -> R
where
    F: FnOnce(&mut FeatureManager) -> R,
{
    let mut guard = FEATURE_MANAGER.lock().expect("FeatureManager lock poisoned");
    match guard.as_mut() {
        Some(mgr) => f(mgr),
        None => default,
    }
}

/// Initialize the Rust core with the data directory path.
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_initialize(
    mut env: JNIEnv,
    _class: JClass,
    data_dir: JString,
) -> jboolean {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("NovaRust"),
    );

    let data_dir_str: String = match jstring_to_string(&mut env, data_dir) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to get data_dir string: {:?}", e);
            return JNI_FALSE;
        }
    };

    let path = std::path::PathBuf::from(&data_dir_str);

    let config = AppConfig::load().unwrap_or_default();
    *APP_CONFIG.lock().unwrap() = Some(config);

    let manager = FeatureManager::new(path);
    *FEATURE_MANAGER.lock().unwrap() = Some(manager);
    log::info!("Nova Android bridge initialized successfully");
    JNI_TRUE
}

/// Get current settings as JSON string.
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_getSettingsJson(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let json = with_manager(|mgr| {
        let settings = mgr.get_settings();
        serde_json::to_string(settings).unwrap_or_else(|_| "{}".to_string())
    });
    env.new_string(json)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Update settings from JSON string.
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_updateSettings(
    mut env: JNIEnv,
    _class: JClass,
    json: JString,
) -> jboolean {
    let json_str: String = match jstring_to_string(&mut env, json) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };

    let settings: nova_features::settings::Settings = match serde_json::from_str(&json_str) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };

    with_manager_opt(JNI_FALSE, |mgr| {
        mgr.update_settings(&settings);
        JNI_TRUE
    })
}

/// Get all bookmarks as JSON string.
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_getBookmarksJson(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let json = with_manager(|mgr| {
        let bookmarks = mgr.get_bookmarks();
        serde_json::to_string(&bookmarks).unwrap_or_else(|_| "[]".to_string())
    });
    env.new_string(json)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Add a bookmark.
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_addBookmark(
    mut env: JNIEnv,
    _class: JClass,
    title: JString,
    url: JString,
) -> jboolean {
    let title_str: String = match jstring_to_string(&mut env, title) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };
    let url_str: String = match jstring_to_string(&mut env, url) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };

    with_manager_opt(JNI_FALSE, |mgr| {
        mgr.add_bookmark(&title_str, &url_str);
        JNI_TRUE
    })
}

/// Remove a bookmark by URL.
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_removeBookmark(
    mut env: JNIEnv,
    _class: JClass,
    url: JString,
) -> jboolean {
    let url_str: String = match jstring_to_string(&mut env, url) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };

    with_manager_opt(JNI_FALSE, |mgr| {
        mgr.remove_bookmark(&url_str);
        JNI_TRUE
    })
}

/// Get browsing history as JSON string.
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_getHistoryJson(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let json = with_manager(|mgr| {
        let history = mgr.get_history();
        serde_json::to_string(&history).unwrap_or_else(|_| "[]".to_string())
    });
    env.new_string(json)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Clear all browsing history.
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_clearHistory(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    with_manager_opt(JNI_FALSE, |mgr| {
        mgr.clear_history();
        JNI_TRUE
    })
}

/// Get saved passwords as JSON string (requires master password for decryption).
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_getPasswordsJson(
    mut env: JNIEnv,
    _class: JClass,
    master_password: JString,
) -> jstring {
    let master: String = match jstring_to_string(&mut env, master_password) {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let json = with_manager(|mgr| {
        let passwords = mgr.get_passwords(&master);
        serde_json::to_string(&passwords).unwrap_or_else(|_| "[]".to_string())
    });
    env.new_string(json)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Save a password entry.
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_savePassword(
    mut env: JNIEnv,
    _class: JClass,
    site: JString,
    username: JString,
    password: JString,
    master_password: JString,
) -> jboolean {
    let site_str: String = match jstring_to_string(&mut env, site) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };
    let username_str: String = match jstring_to_string(&mut env, username) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };
    let password_str: String = match jstring_to_string(&mut env, password) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };
    let master_str: String = match jstring_to_string(&mut env, master_password) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };

    with_manager_opt(JNI_FALSE, |mgr| {
        mgr.save_password(&site_str, &username_str, &password_str, &master_str);
        JNI_TRUE
    })
}

/// Check if a URL should be blocked by the ad blocker.
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_shouldBlockUrl(
    mut env: JNIEnv,
    _class: JClass,
    url: JString,
) -> jboolean {
    let url_str: String = match jstring_to_string(&mut env, url) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };

    with_manager_opt(JNI_FALSE, |mgr| {
        if mgr.should_block_url(&url_str) {
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    })
}

/// Add a custom ad-block filter rule.
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_addAdBlockRule(
    mut env: JNIEnv,
    _class: JClass,
    rule: JString,
) -> jboolean {
    let rule_str: String = match jstring_to_string(&mut env, rule) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };

    with_manager_opt(JNI_FALSE, |mgr| {
        mgr.add_adblock_rule(&rule_str);
        JNI_TRUE
    })
}

/// Get download manager entries as JSON string.
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_getDownloadsJson(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let json = with_manager(|mgr| {
        let downloads = mgr.get_downloads();
        serde_json::to_string(&downloads).unwrap_or_else(|_| "[]".to_string())
    });
    env.new_string(json)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Extract readable content from HTML (reading mode).
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_extractReadableContent(
    mut env: JNIEnv,
    _class: JClass,
    html: JString,
) -> jstring {
    let html_str: String = match jstring_to_string(&mut env, html) {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let content = with_manager(|mgr| mgr.extract_reading_content(&html_str));

    env.new_string(content)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Check URL safety (returns "safe", "phishing", "malware", or "unknown").
#[no_mangle]
pub extern "system" fn Java_com_novabrowser_bridge_RustBridge_checkUrlSafety(
    mut env: JNIEnv,
    _class: JClass,
    url: JString,
) -> jstring {
    let url_str: String = match jstring_to_string(&mut env, url) {
        Ok(s) => s,
        Err(_) => {
            return env.new_string("unknown")
                .map(|s| s.into_raw())
                .unwrap_or(std::ptr::null_mut());
        }
    };

    let result = with_manager(|mgr| mgr.check_url_safety(&url_str));

    env.new_string(result)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}