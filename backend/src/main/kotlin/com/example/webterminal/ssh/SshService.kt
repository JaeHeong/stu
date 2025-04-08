package com.example.webterminal.ssh

import com.corundumstudio.socketio.SocketIOClient
import com.example.webterminal.socketio.service.SshSocketIOService
import jakarta.annotation.PreDestroy
import org.springframework.stereotype.Service

@Service
class SshService(
    private val sshSessionCache: SshSessionCache,
    private val sshSocketIOService: SshSocketIOService
) {

    fun createSshSession(socketIOClient: SocketIOClient) {
        (sshSessionCache.findByUUID(socketIOClient.sessionId)
            ?: SshSession(
                remoteUser = "root", // Change this to your username
                remoteHost = "localhost",
                remotePassword = "password", // Change this to your password
                onByteArrayEvent = {
                    sshSocketIOService.sendUpdateMessage(
                        socketIOClient,
                        it.toString(Charsets.UTF_8)
                    )
                },
                onEofEvent = {
                    sshSocketIOService.sendEofMessage(socketIOClient)
                    sshSocketIOService.disconnect(socketIOClient)
                },
                sessionId = socketIOClient.sessionId
            )).also { sshSessionCache.add(it) }
    }

    fun writeToSsh(socketIOClient: SocketIOClient, data: String) {
        val sshSession = sshSessionCache.findByUUID(socketIOClient.sessionId)

        if (sshSession == null) {
            sshSocketIOService.sendNoSessionMessage(socketIOClient)
        } else {
            if (sshSession.isSessionAlive()) {
                sshSession.write(data)
            } else {
                sshSocketIOService.sendSessionClosedMessage(socketIOClient)
            }
        }
    }

    fun disconnect(socketIOClient: SocketIOClient) {
        sshSessionCache.destroyOne(socketIOClient.sessionId)
    }

    @PreDestroy
    fun destroy() {
        sshSessionCache.destroyAll()
    }
}