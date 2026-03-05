package com.rin.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.Binder
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import com.rin.MainActivity
import com.rin.terminal.SessionManager


class TerminalSessionService : Service() {

    companion object {
        private const val CHANNEL_ID = "rin_terminal_channel"
        private const val NOTIFICATION_ID = 1001
        const val ACTION_STOP = "com.rin.service.ACTION_STOP"
    }

    inner class LocalBinder : Binder() {
        val service: TerminalSessionService get() = this@TerminalSessionService
    }

    private val binder = LocalBinder()
    var sessionManager: SessionManager? = null

    override fun onBind(intent: Intent?): IBinder = binder

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            sessionManager?.destroyAll()
            stopForeground(STOP_FOREGROUND_REMOVE)
            stopSelf()
            return START_NOT_STICKY
        }

        startForeground(NOTIFICATION_ID, buildNotification(1))
        return START_STICKY
    }

    fun updateNotification(sessionCount: Int) {
        val manager = getSystemService(NotificationManager::class.java)
        manager.notify(NOTIFICATION_ID, buildNotification(sessionCount))
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Terminal Sessions",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Keeps terminal sessions alive in the background"
                setShowBadge(false)
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    private fun buildNotification(sessionCount: Int): Notification {
        val openIntent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
        }
        val openPending = PendingIntent.getActivity(
            this, 0, openIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val stopIntent = Intent(this, TerminalSessionService::class.java).apply {
            action = ACTION_STOP
        }
        val stopPending = PendingIntent.getService(
            this, 1, stopIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val sessionLabel = if (sessionCount == 1) "session" else "sessions"

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentTitle("Rin Terminal")
            .setContentText("$sessionCount $sessionLabel running")
            .setOngoing(true)
            .setContentIntent(openPending)
            .addAction(
                android.R.drawable.ic_delete,
                "Exit",
                stopPending
            )
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .build()
    }

    override fun onDestroy() {
        sessionManager?.destroyAll()
        super.onDestroy()
    }
}
