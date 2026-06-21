package com.novabrowser.browser

import android.app.DownloadManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Environment
import android.os.IBinder
import android.util.Log
import org.mozilla.geckoview.GeckoResult
import org.mozilla.geckoview.GeckoRuntime
import org.mozilla.geckoview.GeckoSession
import org.mozilla.geckoview.GeckoSessionSettings
import org.mozilla.geckoview.WebRequestError

/**
 * GeckoView-based browser engine for Android.
 * Manages GeckoRuntime, sessions, and page navigation.
 */
class GeckoViewEngine(private val context: Context) {

    companion object {
        private const val TAG = "GeckoViewEngine"
        private var runtime: GeckoRuntime? = null

        @Synchronized
        fun getRuntime(context: Context): GeckoRuntime {
            if (runtime == null) {
                val settings = GeckoRuntimeSettings.Builder()
                    .aboutConfigEnabled(true)
                    .remoteDebuggingEnabled(true)
                    .consoleOutput(true)
                    .trackingProtection(true)
                    .build()

                runtime = GeckoRuntime.create(context, settings)
                Log.i(TAG, "GeckoRuntime initialized")
            }
            return runtime!!
        }
    }

    data class BrowserSession(
        val id: String,
        val session: GeckoSession,
        var title: String = "",
        var url: String = "",
        var isLoading: Boolean = false,
        var progress: Int = 0
    )

    private val sessions = mutableListOf<BrowserSession>()
    private var activeSessionId: String? = null

    val activeSession: BrowserSession?
        get() = sessions.find { it.id == activeSessionId }

    fun createSession(
        isPrivate: Boolean = false,
        onTitleChanged: (String) -> Unit = {},
        onUrlChanged: (String) -> Unit = {},
        onProgressChanged: (Int) -> Unit = {},
        onLoadingStateChanged: (Boolean) -> Unit = {},
        onError: (String) -> Unit = {}
    ): BrowserSession {
        val runtime = getRuntime(context)

        val settings = GeckoSessionSettings.Builder()
            .usePrivateMode(isPrivate)
            .useTrackingProtection(true)
            .build()

        val session = GeckoSession(settings)
        val id = java.util.UUID.randomUUID().toString()

        val browserSession = BrowserSession(id = id, session = session)

        session.navigationDelegate = object : GeckoSession.NavigationDelegate {
            override fun onLocationChange(
                s: GeckoSession, url: String?, perms: MutableList<GeckoSession.PermissionDelegate.ContentPermission>?
            ) {
                url?.let {
                    browserSession.url = it
                    onUrlChanged(it)
                }
            }

            override fun onLoadRequest(s: GeckoSession, request: GeckoSession.NavigationDelegate.LoadRequest):
                GeckoResult<GeckoSession.NavigationDelegate.LoadRequest.AllowOrDeny>? {
                Log.d(TAG, "Load request: ${request.uri}")
                return GeckoResult.fromValue(GeckoSession.NavigationDelegate.LoadRequest.ALLOW)
            }

            override fun onLoadError(
                s: GeckoSession, uri: String?, error: WebRequestError
            ): GeckoResult<String>? {
                Log.e(TAG, "Load error: ${error.message}")
                onError("${error.category}: ${error.message}")
                return GeckoResult.fromValue(
                    "<html><body><h2>加载失败</h2><p>${error.message}</p></body></html>"
                )
            }

            override fun onCanGoBack(s: GeckoSession, canGoBack: Boolean) {}
            override fun onCanGoForward(s: GeckoSession, canGoForward: Boolean) {}
        }

        session.progressDelegate = object : GeckoSession.ProgressDelegate {
            override fun onPageStart(s: GeckoSession, url: String) {
                browserSession.isLoading = true
                onLoadingStateChanged(true)
            }

            override fun onPageStop(s: GeckoSession, success: Boolean) {
                browserSession.isLoading = false
                onLoadingStateChanged(false)
            }

            override fun onProgressChange(s: GeckoSession, progress: Int) {
                browserSession.progress = progress
                onProgressChanged(progress)
            }

            override fun onSecurityChange(
                s: GeckoSession, securityInfo: GeckoSession.ProgressDelegate.SecurityInformation
            ) {}
        }

        session.contentDelegate = object : GeckoSession.ContentDelegate {
            override fun onTitleChange(s: GeckoSession, title: String?) {
                title?.let {
                    browserSession.title = it
                    onTitleChanged(it)
                }
            }

            override fun onContextMenu(
                s: GeckoSession, screenX: Int, screenY: Int,
                element: GeckoSession.ContentDelegate.ContextElement
            ) {}
        }

        session.open(runtime)
        sessions.add(browserSession)
        return browserSession
    }

    fun loadUrl(sessionId: String, url: String) {
        val session = sessions.find { it.id == sessionId }
        val normalizedUrl = normalizeUrl(url)
        session?.session?.loadUri(normalizedUrl)
        activeSessionId = sessionId
    }

    fun goBack(sessionId: String) {
        sessions.find { it.id == sessionId }?.session?.goBack()
    }

    fun goForward(sessionId: String) {
        sessions.find { it.id == sessionId }?.session?.goForward()
    }

    fun reload(sessionId: String) {
        sessions.find { it.id == sessionId }?.session?.reload()
    }

    fun stop(sessionId: String) {
        sessions.find { it.id == sessionId }?.session?.stop()
    }

    fun closeSession(sessionId: String) {
        val session = sessions.find { it.id == sessionId }
        session?.session?.close()
        sessions.removeAll { it.id == sessionId }
        if (activeSessionId == sessionId) {
            activeSessionId = sessions.lastOrNull()?.id
        }
    }

    fun closeAllSessions() {
        sessions.forEach { it.session.close() }
        sessions.clear()
        activeSessionId = null
    }

    fun downloadFile(sessionId: String, url: String, filename: String) {
        val dm = context.getSystemService(Context.DOWNLOAD_SERVICE) as DownloadManager
        val request = DownloadManager.Request(Uri.parse(url))
            .setTitle(filename)
            .setDescription("Downloading $filename")
            .setNotificationVisibility(DownloadManager.Request.VISIBILITY_VISIBLE_NOTIFY_COMPLETED)
            .setDestinationInExternalPublicDir(Environment.DIRECTORY_DOWNLOADS, filename)
        dm.enqueue(request)
    }

    private fun normalizeUrl(input: String): String {
        return when {
            input.startsWith("http://") || input.startsWith("https://") -> input
            input.contains(".") && !input.contains(" ") -> "https://$input"
            else -> "https://www.google.com/search?q=${java.net.URLEncoder.encode(input, "UTF-8")}"
        }
    }
}

/**
 * Foreground service for managing downloads.
 */
class DownloadService : Service() {
    override fun onBind(intent: Intent?): IBinder? = null
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        return START_NOT_STICKY
    }
}