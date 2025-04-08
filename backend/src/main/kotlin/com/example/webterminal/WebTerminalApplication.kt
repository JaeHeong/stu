package com.example.webterminal

import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.runApplication

@SpringBootApplication
class WebTerminalApplication

fun main(args: Array<String>) {
    runApplication<WebTerminalApplication>(*args)
}