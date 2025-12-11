package io.github.teamclouday.AndroidMic.ui

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.*
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import android.util.Log
import androidx.annotation.RequiresApi
import io.github.teamclouday.AndroidMic.domain.service.*

@RequiresApi(Build.VERSION_CODES.N)
class ConnectTileService : TileService() {

    private val TAG = "ConnectTileService"

    private var serviceMessenger: Messenger? = null
    private var isBound = false
    private var isStreaming = false

    private lateinit var responseMessenger: Messenger
    private lateinit var handlerThread: HandlerThread
    private lateinit var serviceLooper: Looper

    private inner class ResponseHandler(looper: Looper) : Handler(looper) {
        override fun handleMessage(msg: Message) {
            val response = ResponseData.fromMessage(msg)
            isStreaming = response.state == ServiceState.Connected
            updateTile()
        }
    }

    override fun onCreate() {
        super.onCreate()
        handlerThread = HandlerThread("TileServiceResponse")
        handlerThread.start()
        serviceLooper = handlerThread.looper
        responseMessenger = Messenger(ResponseHandler(serviceLooper))
    }

    private val connection = object : ServiceConnection {
        override fun onServiceConnected(className: ComponentName, service: IBinder) {
            Log.d(TAG, "onServiceConnected")
            serviceMessenger = Messenger(service)
            isBound = true
            // Request status update
            sendCommand(Command.GetStatus)
        }

        override fun onServiceDisconnected(arg0: ComponentName) {
            Log.d(TAG, "onServiceDisconnected")
            serviceMessenger = null
            isBound = false
            isStreaming = false
            updateTile()
        }
    }

    override fun onStartListening() {
        super.onStartListening()
        Log.d(TAG, "onStartListening")
        Intent(this, ForegroundService::class.java).also { intent ->
            bindService(intent, connection, Context.BIND_AUTO_CREATE)
        }
    }

    override fun onStopListening() {
        super.onStopListening()
        Log.d(TAG, "onStopListening")
        if (isBound) {
            unbindService(connection)
            isBound = false
        }
    }

    override fun onClick() {
        super.onClick()
        Log.d(TAG, "onClick")
        if (isStreaming) {
            sendCommand(Command.StopStream)
        } else {
            val intent = Intent(this, MainActivity::class.java).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            startActivityAndCollapse(intent)
        }
    }
    
    override fun onDestroy() {
        super.onDestroy()
        serviceLooper.quitSafely()
        try {
            handlerThread.join(500)
        } catch (e: InterruptedException) {
            // ignore
        }
    }

    private fun sendCommand(command: Command, commandData: CommandData? = null) {
        if (!isBound) return
        val msg = Message.obtain(null, command.ordinal)
        msg.replyTo = responseMessenger
        if (commandData != null) {
            msg.data = commandData.toBundle()
        }
        try {
            serviceMessenger?.send(msg)
        } catch (e: RemoteException) {
            Log.e(TAG, "Failed to send message to service", e)
        }
    }

    private fun updateTile() {
        val tile = qsTile
        if (tile != null) {
            tile.state = if (isStreaming) Tile.STATE_ACTIVE else Tile.STATE_INACTIVE
            tile.updateTile()
        }
    }
}
