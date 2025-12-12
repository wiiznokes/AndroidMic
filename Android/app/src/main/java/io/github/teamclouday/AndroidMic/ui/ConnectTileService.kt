package io.github.teamclouday.AndroidMic.ui

import android.content.ComponentName
import android.content.Intent
import android.content.ServiceConnection
import android.os.*
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import android.util.Log
import androidx.annotation.RequiresApi
import io.github.teamclouday.AndroidMic.AppPreferences
import io.github.teamclouday.AndroidMic.Dialogs
import io.github.teamclouday.AndroidMic.domain.service.*
import io.github.teamclouday.AndroidMic.utils.Either
import io.github.teamclouday.AndroidMic.utils.ignore
import kotlinx.coroutines.runBlocking

@RequiresApi(Build.VERSION_CODES.N)
class ConnectTileService : TileService() {

    private val TAG = "ConnectTileService"

    private val prefs = AppPreferences(this)

    private var service: Messenger? = null
    private var isBound = false
    private lateinit var handlerThread: HandlerThread
    private lateinit var messenger: Messenger
    private lateinit var messengerLooper: Looper
    private var isStreamStarted = false
    private var isClickable = true


    private inner class ResponseHandler(looper: Looper) : Handler(looper) {
        override fun handleMessage(msg: Message) {
            val response = ResponseData.fromMessage(msg)

            when (response.kind) {
                Response.Standard -> {
                    response.state?.let {
                        isClickable = true
                        isStreamStarted = it == ServiceState.Connected
                    }
                }
            }
            updateTile()
        }
    }

    override fun onCreate() {
        super.onCreate()
        handlerThread =
            HandlerThread("TileServiceResponseHandler", Process.THREAD_PRIORITY_BACKGROUND)
        handlerThread.start()
        messengerLooper = handlerThread.looper
        messenger = Messenger(ResponseHandler(messengerLooper))

        val intent = Intent(this, ForegroundService::class.java).apply {
            action = BIND_SERVICE_ACTION
        }
        startService(intent)
        bindService(intent, connection, BIND_AUTO_CREATE)
    }

    private val connection = object : ServiceConnection {
        override fun onServiceConnected(className: ComponentName, service: IBinder) {
            Log.d(TAG, "onServiceConnected")
            this@ConnectTileService.service = Messenger(service)
            isBound = true

            requestListeningState(
                this@ConnectTileService,
                ComponentName(this@ConnectTileService, ConnectTileService::class.java)
            )

            val message = Message.obtain()
            message.what = Command.GetStatus.ordinal
            message.replyTo = messenger
            this@ConnectTileService.service?.send(message)
        }

        override fun onServiceDisconnected(arg0: ComponentName) {
            Log.d(TAG, "onServiceDisconnected")
            service = null
            isBound = false
            isStreamStarted = false
            isClickable = false
            updateTile()
        }
    }

    override fun onStartListening() {
        super.onStartListening()
        Log.d(TAG, "onStartListening")
    }

    override fun onStopListening() {
        super.onStopListening()
        Log.d(TAG, "onStopListening")
    }

    override fun onClick() {
        super.onClick()
        Log.d(TAG, "onClick")

        if (!isBound) {
            Log.d(TAG, "service not bound, can't send command")
            return
        }

        if (service == null) {
            Log.d(TAG, "service is null, can't send command")
            return
        }

        val reply = if (isStreamStarted) {
            CommandData(Command.StopStream)
        } else {
            val res = runBlocking {
                CommandData.fromPref(prefs, Command.StartStream)
            }
            when (res) {
                is Either.Left<CommandData> -> {
                    res.value
                }

                is Either.Right<Dialogs> -> {
                    Log.d(TAG, "missing parameter to start stream")
                    return
                }
            }
        }.toCommandMsg()

        isClickable = false
        reply.replyTo = messenger
        service?.send(reply)
    }

    override fun onDestroy() {
        super.onDestroy()

        messengerLooper.quitSafely()
        ignore { handlerThread.join(WAIT_PERIOD) }
        if (isBound) {
            unbindService(connection)
            isBound = false
        }
    }

    private fun updateTile() {
        val tile = qsTile
        if (tile != null) {
            tile.state = if (isStreamStarted) Tile.STATE_ACTIVE else Tile.STATE_INACTIVE
            tile.updateTile()
        }
    }
}
