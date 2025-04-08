package com.example.webterminal.config

import com.corundumstudio.socketio.SocketConfig
import com.corundumstudio.socketio.SocketIOServer
import org.springframework.context.annotation.Bean
import org.springframework.context.annotation.Configuration

@Configuration
class SocketIOConfig {
    @Bean
    fun socketIOServer(): SocketIOServer {
        return com.corundumstudio.socketio.Configuration().apply {
            hostname = "localhost"
            port = 8081
            socketConfig = SocketConfig().apply {
                isReuseAddress = true
            }
        }.let { SocketIOServer(it) }
    }
}