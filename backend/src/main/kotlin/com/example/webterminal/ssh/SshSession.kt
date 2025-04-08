package com.example.webterminal.ssh

import com.jcraft.jsch.ChannelShell
import com.jcraft.jsch.JSch
import com.jcraft.jsch.Session
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.io.InputStream
import java.io.OutputStream
import java.util.*

class SshSession(
    private val remoteUser: String,
    private val remoteHost: String,
    private val remotePassword: String,
    private val remotePort: Int = 22,
    val sessionId: UUID,
    private val onByteArrayEvent: (ByteArray) -> Unit,
    private val onEofEvent: () -> Unit
) {
    private val jSch: JSch = JSch()

    private lateinit var remoteSession: Session
    private lateinit var channel: ChannelShell
    private lateinit var out: OutputStream

    init {
        createSession()
        connect()
    }

    private fun createSession() {
        remoteSession = jSch.getSession(remoteUser, remoteHost, remotePort)
    }

    private fun connect() {
        remoteSession.run {
            setPassword(remotePassword)
            setConfig("StrictHostKeyChecking", "no")
            connect()
        }

        channel = remoteSession.openChannel("shell") as ChannelShell
        out = channel.outputStream

        channel.connect()

        write("/bin/bash\n")
        startChannel(channel.inputStream)
    }

    private fun startChannel(out: InputStream) {
        CoroutineScope(Dispatchers.IO).launch {
            val buffer = ByteArray(1024)
            try {
                while (true) {
                    val bufferSize = out.read(buffer)
                    if (bufferSize < 0) {
                        break
                    } else {
                        onByteArrayEvent(buffer.copyOf(bufferSize))
                    }
                }
            } finally {
                onEofEvent()
            }
        }
    }

    fun write(data: String) {
        val bytes = data.toByteArray(Charsets.UTF_8)
        out.write(bytes)
        out.flush()
    }

    fun disconnect() {
        channel.disconnect()
        remoteSession.disconnect()
    }

    fun isSessionAlive(): Boolean {
        return remoteSession.isConnected && channel.isConnected
    }
}