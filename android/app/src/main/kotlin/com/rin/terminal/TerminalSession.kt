package com.rin.terminal

import java.util.UUID

data class TerminalSession(
    val id: String = UUID.randomUUID().toString(),
    var name: String,
    var engineHandle: Long,
    val isRoot: Boolean = false
)
