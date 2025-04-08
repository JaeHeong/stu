package com.example.webterminal.socketio.service

import com.corundumstudio.socketio.SocketIOClient
import org.springframework.stereotype.Service

@Service
class SshSocketIOService {
    fun sendUpdateMessage(socketIOClient: SocketIOClient, message: String) {
        socketIOClient.sendEvent("update", message)
    }

    fun sendEofMessage(socketIOClient: SocketIOClient) {
        socketIOClient.sendEvent("eof")
    }

    fun sendNoSessionMessage(socketIOClient: SocketIOClient) {
        socketIOClient.sendEvent("error", "No session found")
    }

    fun sendSessionClosedMessage(socketIOClient: SocketIOClient) {
        socketIOClient.sendEvent("error", "Session closed")
    }
    
    fun sendSessionCreateErrorMessage(socketIOClient: SocketIOClient) {
        socketIOClient.sendEvent("error", "Failed to create SSH session")
    }

    fun disconnect(socketIOClient: SocketIOClient) {
        socketIOClient.disconnect()
    }
}