package com.example.webterminal.socketio

import com.corundumstudio.socketio.SocketIOClient
import com.corundumstudio.socketio.SocketIOServer
import com.corundumstudio.socketio.listener.ConnectListener
import com.corundumstudio.socketio.listener.DataListener
import com.corundumstudio.socketio.listener.DisconnectListener
import com.example.webterminal.socketio.service.SshSocketIOService
import com.example.webterminal.ssh.SshService
import io.github.oshai.kotlinlogging.KotlinLogging
import org.springframework.stereotype.Component

private val log = KotlinLogging.logger {}

@Component
class SshSocketIOModule(
    socketIOServer: SocketIOServer,
    private val sshService: SshService,
    private val sshSocketIOService: SshSocketIOService
) {

    init {
        socketIOServer.addNamespace("/ssh").apply {
            addConnectListener(onConnectListener())
            addDisconnectListener(onDisconnectListener())
            addEventListener("connectTerminal", Void::class.java, onConnectTerminalListener())
            addEventListener("type", String::class.java, onTypeListener())
        }
    }

    private fun onConnectListener(): ConnectListener {
        return ConnectListener { client ->
            log.info { "Client Connected: ${client.sessionId}" }
        }
    }

    private fun onDisconnectListener(): DisconnectListener {
        return DisconnectListener { client ->
            log.info { "Client Disconnected: ${client.sessionId}" }
            sshService.disconnect(client)
        }
    }

    private fun onConnectTerminalListener(): DataListener<Void> {
        return DataListener { client, _, _ ->
            log.info { "SessionId: ${client.sessionId} Event: connectTerminal" }
            try {
                sshService.createSshSession(client)
            } catch (e: Exception) {
                log.error(e) { "Error creating SSH session" }
                sshSocketIOService.sendSessionCreateErrorMessage(client)
            }
        }
    }

    private fun onTypeListener(): DataListener<String> {
        return DataListener { client, data, _ ->
            log.info { "SessionId: ${client.sessionId} Event: type Data: $data" }
            sshService.writeToSsh(client, data)
        }
    }
}