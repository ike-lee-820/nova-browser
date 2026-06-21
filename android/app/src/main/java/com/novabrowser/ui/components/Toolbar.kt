package com.novabrowser.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NovaToolbar(
    url: String,
    onUrlChange: (String) -> Unit,
    onUrlSubmit: (String) -> Unit,
    onBack: () -> Unit,
    onForward: () -> Unit,
    onReload: () -> Unit,
    onStop: () -> Unit,
    onMenu: () -> Unit,
    isLoading: Boolean,
    progress: Int,
    modifier: Modifier = Modifier
) {
    Column(modifier = modifier) {
        // Progress bar
        if (isLoading) {
            LinearProgressIndicator(
                progress = { progress / 100f },
                modifier = Modifier.fillMaxWidth().height(3.dp),
                color = MaterialTheme.colorScheme.primary,
                trackColor = MaterialTheme.colorScheme.surfaceVariant,
            )
        }

        // Toolbar row
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 4.dp, vertical = 2.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(2.dp)
        ) {
            // Back button
            IconButton(onClick = onBack) {
                Icon(
                    Icons.AutoMirrored.Filled.ArrowBack,
                    contentDescription = "后退",
                    tint = MaterialTheme.colorScheme.onSurface
                )
            }

            // Forward button
            IconButton(onClick = onForward) {
                Icon(
                    Icons.AutoMirrored.Filled.ArrowForward,
                    contentDescription = "前进",
                    tint = MaterialTheme.colorScheme.onSurface
                )
            }

            // Reload/Stop button
            IconButton(onClick = { if (isLoading) onStop() else onReload() }) {
                Icon(
                    if (isLoading) Icons.Default.Close else Icons.Default.Refresh,
                    contentDescription = if (isLoading) "停止" else "刷新",
                    tint = MaterialTheme.colorScheme.onSurface
                )
            }

            // URL bar
            OutlinedTextField(
                value = url,
                onValueChange = onUrlChange,
                modifier = Modifier.weight(1f).height(48.dp),
                singleLine = true,
                placeholder = { Text("搜索或输入网址") },
                colors = OutlinedTextFieldDefaults.colors(
                    focusedBorderColor = MaterialTheme.colorScheme.primary,
                    unfocusedBorderColor = MaterialTheme.colorScheme.outline,
                    focusedContainerColor = MaterialTheme.colorScheme.surface,
                    unfocusedContainerColor = MaterialTheme.colorScheme.surfaceVariant,
                ),
                shape = MaterialTheme.shapes.medium,
            ) {
                onUrlSubmit(url)
            }

            // Menu button
            IconButton(onClick = onMenu) {
                Icon(
                    Icons.Default.MoreVert,
                    contentDescription = "菜单",
                    tint = MaterialTheme.colorScheme.onSurface
                )
            }
        }
    }
}

@Composable
fun NovaTabBar(
    tabCount: Int,
    onNewTab: () -> Unit,
    onSwitchTab: (Int) -> Unit,
    onCloseTab: (Int) -> Unit,
    activeTabIndex: Int,
    modifier: Modifier = Modifier
) {
    if (tabCount <= 1) return

    Row(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 4.dp),
        horizontalArrangement = Arrangement.Start
    ) {
        repeat(tabCount) { index ->
            SuggestionChip(
                onClick = { onSwitchTab(index) },
                label = { Text("标签 ${index + 1}") },
                modifier = Modifier.padding(2.dp),
                colors = SuggestionChipDefaults.suggestionChipColors(
                    containerColor = if (index == activeTabIndex)
                        MaterialTheme.colorScheme.primaryContainer
                    else
                        MaterialTheme.colorScheme.surfaceVariant
                ),
                trailingIcon = {
                    if (tabCount > 1) {
                        IconButton(
                            onClick = { onCloseTab(index) },
                            modifier = Modifier.size(16.dp)
                        ) {
                            Icon(Icons.Default.Close, contentDescription = "关闭", modifier = Modifier.size(12.dp))
                        }
                    }
                }
            )
        }
    }
}

@Composable
fun NovaMenu(
    expanded: Boolean,
    onDismiss: () -> Unit,
    onNewTab: () -> Unit,
    onNewPrivateTab: () -> Unit,
    onBookmarks: () -> Unit,
    onHistory: () -> Unit,
    onDownloads: () -> Unit,
    onSettings: () -> Unit,
    onExtensions: () -> Unit,
    onShare: () -> Unit,
    onFindInPage: () -> Unit,
    onDesktopSite: () -> Unit,
    onReaderMode: () -> Unit,
) {
    DropdownMenu(expanded = expanded, onDismissRequest = onDismiss) {
        DropdownMenuItem(text = { Text("新标签页") }, onClick = {
            onNewTab(); onDismiss()
        })
        DropdownMenuItem(text = { Text("隐私标签页") }, onClick = {
            onNewPrivateTab(); onDismiss()
        })
        HorizontalDivider()
        DropdownMenuItem(text = { Text("书签") }, onClick = {
            onBookmarks(); onDismiss()
        })
        DropdownMenuItem(text = { Text("历史记录") }, onClick = {
            onHistory(); onDismiss()
        })
        DropdownMenuItem(text = { Text("下载") }, onClick = {
            onDownloads(); onDismiss()
        })
        HorizontalDivider()
        DropdownMenuItem(text = { Text("扩展") }, onClick = {
            onExtensions(); onDismiss()
        })
        DropdownMenuItem(text = { Text("设置") }, onClick = {
            onSettings(); onDismiss()
        })
        HorizontalDivider()
        DropdownMenuItem(text = { Text("分享") }, onClick = {
            onShare(); onDismiss()
        })
        DropdownMenuItem(text = { Text("在页面中查找") }, onClick = {
            onFindInPage(); onDismiss()
        })
        DropdownMenuItem(text = { Text("桌面版网站") }, onClick = {
            onDesktopSite(); onDismiss()
        })
        DropdownMenuItem(text = { Text("阅读模式") }, onClick = {
            onReaderMode(); onDismiss()
        })
    }
}