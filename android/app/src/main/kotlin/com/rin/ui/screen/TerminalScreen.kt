package com.rin.ui.screen

import android.app.Activity
import android.view.View
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import com.rin.RinLib
import com.rin.terminal.SessionManager
import com.rin.ui.components.ExtraKeysBar
import com.rin.ui.components.HelpDialog
import com.rin.ui.components.SessionDialog
import com.rin.ui.components.TerminalSurface

@Composable
fun TerminalScreen(
    sessionManager: SessionManager,
    modifier: Modifier = Modifier
) {
    var ctrlPressed by remember { mutableStateOf(false) }
    var keyRepeating by remember { mutableStateOf(false) }
    var terminalView by remember { mutableStateOf<View?>(null) }
    var showSessionDialog by remember { mutableStateOf(false) }
    var showHelpDialog by remember { mutableStateOf(false) }
    var inputBuffer by remember { mutableStateOf("") }

    val context = LocalContext.current
    val activeSession = sessionManager.activeSession
    val engineHandle = activeSession?.engineHandle ?: 0L

    Column(
        modifier = modifier
            .fillMaxSize()
            .systemBarsPadding()
            .imePadding()
    ) {
        TerminalSurface(
            engineHandle = engineHandle,
            ctrlPressed = ctrlPressed,
            cursorBlinkEnabled = !keyRepeating,
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f),
            onInput = { data ->
                if (engineHandle != 0L) {
                    val input = String(data, Charsets.UTF_8)

                    // Update buffer
                    if (input.contains("\r") || input.contains("\n")) {
                        // Enter check the command
                        val command = inputBuffer.trim().lowercase()
                        
                        if (command == "help") {
                            showHelpDialog = true
                            inputBuffer = ""
                            return@TerminalSurface
                        }
                        
                        if (command == "exit" || command == "quit") {
                            (context as? Activity)?.finishAffinity()
                            return@TerminalSurface
                        }
                        // Reset buffer
                        inputBuffer = ""
                    } else if (input.contains("\u007F") || input.contains("\b")) {
                        if (inputBuffer.isNotEmpty()) {
                            inputBuffer = inputBuffer.dropLast(1)
                        }
                    } else if (input.all { it.isLetterOrDigit() || it.isWhitespace() || it in "!@#$%^&*()_+-=[]{}|;':\",./<>?" }) {
                        inputBuffer += input
                    }
                    RinLib.write(engineHandle, data)
                }
            },
            onViewReady = { view -> terminalView = view }
        )

        ExtraKeysBar(
            onKeyPress = { code ->
                if (engineHandle != 0L) {
                    RinLib.write(engineHandle, code.toByteArray())
                    terminalView?.invalidate()
                }
            },
            onCtrlToggle = { active ->
                ctrlPressed = active
            },
            onPaste = {
                // Trigger paste
                (terminalView as? com.rin.ui.components.TerminalCanvasView)?.let { view ->
                    val clipboardManager = context.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                    val clip = clipboardManager.primaryClip
                    if (clip != null && clip.itemCount > 0) {
                        val text = clip.getItemAt(0).text?.toString()
                        if (text != null && engineHandle != 0L) {
                            RinLib.write(engineHandle, text.toByteArray())
                            terminalView?.invalidate()
                        }
                    }
                }
            },
            onRepeatStateChange = { repeating ->
                keyRepeating = repeating
            },
            sessionName = activeSession?.name ?: "No Session",
            onSessionButtonClick = { showSessionDialog = true },
            modifier = Modifier.fillMaxWidth()
        )
    }

    if (showSessionDialog) {
        SessionDialog(
            sessions = sessionManager.sessions,
            activeIndex = sessionManager.activeIndex,
            onDismiss = { showSessionDialog = false },
            onSwitchSession = { index ->
                sessionManager.switchSession(index)
            },
            onCreateSession = {
                sessionManager.createSession()
            },
            onRemoveSession = { index ->
                sessionManager.removeSession(index)
            },
            onRenameSession = { index, newName ->
                sessionManager.renameSession(index, newName)
            }
        )
    }

    if (showHelpDialog) {
        HelpDialog(
            onDismiss = { showHelpDialog = false }
        )
    }
}
