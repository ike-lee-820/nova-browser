package com.novabrowser.bridge

import android.util.Log

/**
 * JNI bridge to the Rust nova-android library.
 * Provides access to bookmarks, history, settings, password manager, etc.
 */
class RustBridge {

    companion object {
        private const val TAG = "RustBridge"
        private var isLoaded = false

        init {
            try {
                System.loadLibrary("nova_android")
                isLoaded = true
                Log.i(TAG, "nova_android native library loaded")
            } catch (e: UnsatisfiedLinkError) {
                Log.w(TAG, "Failed to load nova_android: ${e.message}")
            }
        }
    }

    fun isAvailable(): Boolean = isLoaded

    // --- Native methods ---

    /** Initialize the Rust core with data directory path */
    external fun initialize(dataDir: String): Boolean

    /** Get the current settings as JSON string */
    external fun getSettingsJson(): String

    /** Update settings from JSON string */
    external fun updateSettings(json: String): Boolean

    /** Get all bookmarks as JSON string */
    external fun getBookmarksJson(): String

    /** Add a bookmark */
    external fun addBookmark(title: String, url: String): Boolean

    /** Remove a bookmark by URL */
    external fun removeBookmark(url: String): Boolean

    /** Get browsing history as JSON string */
    external fun getHistoryJson(): String

    /** Clear all browsing history */
    external fun clearHistory(): Boolean

    /** Get saved passwords (decrypted) as JSON string */
    external fun getPasswordsJson(masterPassword: String): String

    /** Save a password entry */
    external fun savePassword(site: String, username: String, password: String, masterPassword: String): Boolean

    /** Check if ad blocker is enabled for a URL */
    external fun shouldBlockUrl(url: String): Boolean

    /** Add a custom ad-block filter rule */
    external fun addAdBlockRule(rule: String): Boolean

    /** Get download manager entries as JSON string */
    external fun getDownloadsJson(): String

    /** Extract readable content from HTML (reading mode) */
    external fun extractReadableContent(html: String): String

    /** Check URL safety (phishing/malware) */
    external fun checkUrlSafety(url: String): String
}