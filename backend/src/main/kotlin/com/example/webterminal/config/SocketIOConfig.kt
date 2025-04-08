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
            // hostname = "localhost"  // <-- 이렇게 주석 처리하거나 라인 삭제
            port = 9090
            socketConfig = SocketConfig().apply {
                isReuseAddress = true
            }
        }.let { SocketIOServer(it) }
    }
}