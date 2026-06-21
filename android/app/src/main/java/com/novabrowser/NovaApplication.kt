package com.novabrowser

import android.app.Application
import android.util.Log
import com.novabrowser.bridge.RustBridge

class NovaApplication : Application() {

    companion object {
        private const val TAG = "NovaApplication"
        lateinit var instance: NovaApplication
            private set
    }

    val rustBridge: RustBridge by lazy { RustBridge() }

    override fun onCreate() {
        super.onCreate()
        instance = this

        // Initialize Rust native library
        try {
            rustBridge.initialize(dataDir.absolutePath)
            Log.i(TAG, "Rust bridge initialized successfully")
        } catch (e: Exception) {
            Log.w(TAG, "Failed to load native library: ${e.message}")
        }
    }
}