package com.example.webterminal.config

import com.corundumstudio.socketio.SocketIOServer
import org.springframework.boot.CommandLineRunner
import org.springframework.stereotype.Component

@Component
class SocketIOServerRunner(private val socketIOServer: SocketIOServer) : CommandLineRunner {
    override fun run(vararg args: String?) {
        socketIOServer.start()
    }
}