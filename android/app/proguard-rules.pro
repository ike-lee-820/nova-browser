# Nova Browser ProGuard Rules

# Keep GeckoView
-keep class org.mozilla.geckoview.** { *; }

# Keep JNI bridge
-keep class com.novabrowser.bridge.RustBridge { *; }

# Keep native methods
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep Compose
-dontwarn androidx.compose.**
-keep class androidx.compose.** { *; }

# Keep Kotlin coroutines
-keepnames class kotlinx.coroutines.internal.MainDispatcherFactory {}
-keepnames class kotlinx.coroutines.CoroutineExceptionHandler {}
-keepclassmembers class kotlinx.coroutines.** {
    volatile <fields>;
}