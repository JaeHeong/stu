package com.example.webterminal.ssh

import com.github.benmanes.caffeine.cache.Cache
import com.github.benmanes.caffeine.cache.Caffeine
import com.github.benmanes.caffeine.cache.RemovalListener
import io.github.oshai.kotlinlogging.KotlinLogging
import org.springframework.stereotype.Component
import java.util.*
import java.util.concurrent.TimeUnit

private val log = KotlinLogging.logger {}

@Component
class SshSessionCache {
    private val removalListener: RemovalListener<UUID, SshSession> =
        RemovalListener { uuid, sshSession, removalCause ->
            log.info { "Session removed: $uuid Cause: $removalCause" }
            sshSession?.disconnect()
        }

    private val sessionCache: Cache<UUID, SshSession> =
        Caffeine.newBuilder()
            .expireAfterAccess(1, TimeUnit.HOURS)
            .removalListener(removalListener)
            .build()

    fun add(sshSession: SshSession) {
        sessionCache.put(sshSession.sessionId, sshSession)
    }

    fun findByUUID(uuid: UUID): SshSession? {
        return sessionCache.getIfPresent(uuid)
    }

    fun destroyOne(uuid: UUID) {
        sessionCache.invalidate(uuid)
    }

    fun destroyAll() {
        sessionCache.invalidateAll()
        sessionCache.cleanUp()
    }
}