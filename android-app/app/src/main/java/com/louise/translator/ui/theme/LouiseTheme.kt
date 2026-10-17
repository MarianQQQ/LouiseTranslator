package com.louise.translator.ui.theme

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily

/**
 * Authentic Louise Translator Design Tokens matching `ui/appwindow.slint`.
 */
object LouiseTheme {
    // Surface & Background Colors
    val Background = Color(0xFF101010)
    val SurfaceCard = Color(0xFF1C1C1C)
    val BorderSubtle = Color(0xFF282828)
    val BorderFocused = Color(0xFFF1FFFF)

    // Text & Accent Colors
    val TextPrimary = Color(0xFFF1FFFF)
    val TextMuted = Color(0xFFA1A1AA)
    val TextDim = Color(0xFF555555)
    val CyanAccent = Color(0xFF00FFFF)
    val DeepLBlue = Color(0xFF00C0F0)

    // Button states
    val ButtonSecNormal = Color(0xFF1A1A1A)
    val ButtonSecHover = Color(0xFF262626)
    val ButtonSecBorder = Color(0xFF333333)

    // Action button primary (Paste as replacement)
    val ButtonPriNormal = Color(0xFFF1FFFF)
    val ButtonPriText = Color(0xFF0F0F12)

    // Fonts
    val MonospaceFont = FontFamily.Monospace // JetBrains Mono fallback
}
