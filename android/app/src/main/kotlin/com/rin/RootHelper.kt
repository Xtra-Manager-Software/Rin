package com.rin

import com.topjohnwu.superuser.Shell
import java.io.File

object RootHelper {
    
    init {
        Shell.enableVerboseLogging = false
        Shell.setDefaultBuilder(
            Shell.Builder.create()
                .setFlags(Shell.FLAG_REDIRECT_STDERR)
                .setTimeout(10)
        )
    }
    
    fun isRootAvailable(): Boolean {
        return try {
            Shell.getShell().isRoot
        } catch (e: Exception) {
            false
        }
    }
    
    fun requestRoot(): Boolean {
        return try {
            val shell = Shell.getShell()
            shell.isRoot
        } catch (e: Exception) {
            false
        }
    }
    
    fun getSuPath(): String? {
        val possiblePaths = listOf(
            "/system/xbin/su",
            "/system/bin/su",
            "/system/sbin/su",
            "/sbin/su",
            "/su/bin/su",
            "/magisk/.core/bin/su",
            "/data/adb/ksu/bin/ksud",
            "/data/adb/ap/bin/su",
            "/data/adb/magisk/busybox"
        )
        
        for (path in possiblePaths) {
            val file = File(path)
            if (file.exists() && file.canExecute()) {
                return path
            }
        }
        
        return Shell.getShell().takeIf { it.isRoot }?.let {
            val result = it.newJob().add("which su").exec()
            if (result.isSuccess && result.out.isNotEmpty()) {
                result.out[0]
            } else {
                null
            }
        }
    }
}
