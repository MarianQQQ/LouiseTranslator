package com.louise.translator.domain

/**
 * Core models for translation and language capabilities.
 * Perfectly mirrors contracts defined in `docs/migration/02-contract.md`.
 */

enum class EngineType(val label: String) {
    DEEPL("DeepL ⚡"),
    GOOGLE("Google")
}

data class TranslationResult(
    val translatedText: String,
    val detectedSourceLang: String?,
    val engineUsed: EngineType,
    val fellBack: Boolean
)

data class DeepLUsage(
    val characterCount: Long,
    val characterLimit: Long
) {
    val remaining: Long get() = (characterLimit - characterCount).coerceAtLeast(0)
    val fraction: Float get() = if (characterLimit > 0) remaining.toFloat() / characterLimit.toFloat() else 0f
    val percent: Float get() = fraction * 100f
}

data class AppConfig(
    val srcLang: String = "auto",
    val dstLang: String = "uk",
    val priorityLangs: List<String> = listOf("uk", "en", "pl", "de", "fr", "es"),
    val useDeepL: Boolean = true,
    val deepLKey: String = "",
    val useEdgeTts: Boolean = true,
    val autoCopy: Boolean = false,
    val closeOnPaste: Boolean = true,
    val uiLang: String = "uk"
)
