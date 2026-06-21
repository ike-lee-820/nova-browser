package com.novabrowser.ui

import android.view.ViewGroup
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import com.novabrowser.browser.GeckoViewEngine
import com.novabrowser.ui.components.NovaMenu
import com.novabrowser.ui.components.NovaTabBar
import com.novabrowser.ui.components.NovaToolbar
import org.mozilla.geckoview.GeckoSession
import org.mozilla.geckoview.GeckoView

@Composable
fun MainScreen(initialUrl: String? = null) {
    val context = LocalContext.current
    val engine = remember { GeckoViewEngine(context) }

    var currentUrl by remember { mutableStateOf("") }
    var pageTitle by remember { mutableStateOf("") }
    var isLoading by remember { mutableStateOf(false) }
    var progress by remember { mutableIntStateOf(0) }
    var showMenu by remember { mutableStateOf(false) }
    var activeTabIndex by remember { mutableIntStateOf(0) }
    var tabCount by remember { mutableIntStateOf(1) }
    var isPrivateMode by remember { mutableStateOf(false) }

    // Create initial session
    val session = remember {
        engine.createSession(
            isPrivate = isPrivateMode,
            onTitleChanged = { pageTitle = it },
            onUrlChanged = { currentUrl = it },
            onProgressChanged = { progress = it },
            onLoadingStateChanged = { isLoading = it },
            onError = { /* handle error */ }
        )
    }

    // Load initial URL if provided
    LaunchedEffect(initialUrl) {
        initialUrl?.let { engine.loadUrl(session.id, it) }
    }

    Column(modifier = Modifier.fillMaxSize()) {
        // Toolbar
        NovaToolbar(
            url = currentUrl,
            onUrlChange = { currentUrl = it },
            onUrlSubmit = { engine.loadUrl(session.id, it) },
            onBack = { engine.goBack(session.id) },
            onForward = { engine.goForward(session.id) },
            onReload = { engine.reload(session.id) },
            onStop = { engine.stop(session.id) },
            onMenu = { showMenu = true },
            isLoading = isLoading,
            progress = progress
        )

        // Tab bar
        NovaTabBar(
            tabCount = tabCount,
            activeTabIndex = activeTabIndex,
            onNewTab = {
                tabCount++
                activeTabIndex = tabCount - 1
            },
            onSwitchTab = { activeTabIndex = it },
            onCloseTab = { index ->
                if (tabCount > 1) {
                    tabCount--
                    if (activeTabIndex >= tabCount) activeTabIndex = tabCount - 1
                }
            }
        )

        // Web content - GeckoView
        Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
            AndroidView(
                factory = { ctx ->
                    GeckoView(ctx).apply {
                        layoutParams = ViewGroup.LayoutParams(
                            ViewGroup.LayoutParams.MATCH_PARENT,
                            ViewGroup.LayoutParams.MATCH_PARENT
                        )
                        setSession(session.session)
                    }
                },
                modifier = Modifier.fillMaxSize()
            )
        }
    }

    // Menu dropdown
    Box {
        NovaMenu(
            expanded = showMenu,
            onDismiss = { showMenu = false },
            onNewTab = {
                tabCount++
                activeTabIndex = tabCount - 1
                val newSession = engine.createSession(
                    isPrivate = false,
                    onTitleChanged = { pageTitle = it },
                    onUrlChanged = { currentUrl = it },
                    onProgressChanged = { progress = it },
                    onLoadingStateChanged = { isLoading = it }
                )
                engine.loadUrl(newSession.id, "about:blank")
            },
            onNewPrivateTab = {
                tabCount++
                activeTabIndex = tabCount - 1
                isPrivateMode = true
                val newSession = engine.createSession(
                    isPrivate = true,
                    onTitleChanged = { pageTitle = it },
                    onUrlChanged = { currentUrl = it },
                    onProgressChanged = { progress = it },
                    onLoadingStateChanged = { isLoading = it }
                )
                engine.loadUrl(newSession.id, "about:blank")
            },
            onBookmarks = { /* TODO */ },
            onHistory = { /* TODO */ },
            onDownloads = { /* TODO */ },
            onSettings = { /* TODO */ },
            onExtensions = { /* TODO */ },
            onShare = { /* TODO */ },
            onFindInPage = { /* TODO */ },
            onDesktopSite = { /* TODO */ },
            onReaderMode = { /* TODO */ },
        )
    }
}