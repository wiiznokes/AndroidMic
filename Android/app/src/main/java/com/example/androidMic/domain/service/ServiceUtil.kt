package com.example.androidMic.domain.service


import android.app.Notification
import android.app.PendingIntent
import android.content.Intent
import android.widget.Toast
import androidx.core.app.NotificationCompat
import com.example.androidMic.R
import com.example.androidMic.ui.MainActivity
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch


const val CHANNEL_ID = "Service"

class MessageUi(private val ctx: ForegroundService) {
    // show message on UI
    fun showMessage(message: String) {
        CoroutineScope(Dispatchers.Main).launch {
            Toast.makeText(ctx, message, Toast.LENGTH_SHORT).show()
        }
    }


    fun getNotification(): Notification {
        // launch activity
        val launchIntent = Intent(ctx, MainActivity::class.java).apply {
            flags =
                (Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_BROUGHT_TO_FRONT)
        }
        val pLaunchIntent =
            PendingIntent.getActivity(ctx, 0, launchIntent, PendingIntent.FLAG_IMMUTABLE)


        val builder = NotificationCompat.Builder(ctx, CHANNEL_ID)
            .setSmallIcon(R.mipmap.ic_launcher)
            .setContentTitle(ctx.getString(R.string.app_name))
            .setContentText(ctx.getString(R.string.notification_info))
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .setContentIntent(pLaunchIntent)
            .setOngoing(true)

        return builder.build()
    }
}

