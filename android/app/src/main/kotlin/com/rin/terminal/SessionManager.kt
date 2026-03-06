package com.rin.terminal

import android.content.Context
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import com.rin.RinLib
import com.rin.RootHelper
import com.rin.permission.StoragePermissionHelper


class SessionManager(
    private val context: Context,
    private val homeDir: String,
    private val username: String
) {
    val sessions = mutableStateListOf<TerminalSession>()
    val activeIndexState = mutableIntStateOf(0)

    val activeIndex: Int get() = activeIndexState.intValue

    val activeSession: TerminalSession?
        get() = sessions.getOrNull(activeIndexState.intValue)

    private var sessionCounter = 0

    fun createSession(asRoot: Boolean = false): TerminalSession {
        sessionCounter++
        val hasPermission = if (StoragePermissionHelper.isStoragePermissionGranted(context)) 1 else 0
        
        val handle = if (asRoot && RootHelper.isRootAvailable()) {
            val suPath = RootHelper.getSuPath() ?: "su"
            RinLib.createRootEngine(
                80, 24, 14.0f,
                homeDir,
                username,
                hasPermission,
                suPath
            )
        } else {
            RinLib.createEngine(
                80, 24, 14.0f,
                homeDir,
                username,
                hasPermission
            )
        }
        
        val session = TerminalSession(
            name = if (asRoot) "Root $sessionCounter" else "Session $sessionCounter",
            engineHandle = handle,
            isRoot = asRoot && RootHelper.isRootAvailable()
        )
        sessions.add(session)
        activeIndexState.intValue = sessions.size - 1
        return session
    }

    fun switchSession(index: Int) {
        if (index in sessions.indices) {
            activeIndexState.intValue = index
        }
    }

    fun removeSession(index: Int) {
        if (index !in sessions.indices) return
        val session = sessions[index]

        if (session.engineHandle != 0L) {
            RinLib.destroyEngine(session.engineHandle)
        }
        sessions.removeAt(index)

        if (sessions.isEmpty()) {
            createSession()
        } else {
            activeIndexState.intValue = activeIndexState.intValue.coerceIn(0, sessions.size - 1)
        }
    }

    fun renameSession(index: Int, newName: String) {
        if (index in sessions.indices) {
            sessions[index] = sessions[index].copy(name = newName)
        }
    }

    fun destroyAll() {
        sessions.forEach { session ->
            if (session.engineHandle != 0L) {
                RinLib.destroyEngine(session.engineHandle)
            }
        }
        sessions.clear()
    }

    val sessionCount: Int get() = sessions.size
}
