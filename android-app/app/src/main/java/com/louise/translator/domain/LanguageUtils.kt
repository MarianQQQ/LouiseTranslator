package com.louise.translator.domain

/**
 * Idiomatic, pure-logic utilities corresponding 1:1 with `src/translate.rs`.
 */
object LanguageUtils {

    /**
     * Post-processing: restores terminal punctuation and initial letter casing
     * occasionally dropped or altered by external translation APIs.
     * Mirrors `fix_translation_punctuation` in `src/translate.rs:6-51`.
     */
    fun fixTranslationPunctuation(src: String, translated: String): String {
        if (translated.isEmpty()) return translated

        val srcTrimmed = src.trim()
        var result = translated.trim()
        if (result.isEmpty()) return result

        // 1. Restore capitalized initial letter if original began with uppercase
        val firstSrcChar = srcTrimmed.firstOrNull()
        if (firstSrcChar != null && firstSrcChar.isUpperCase()) {
            result = result.replaceFirstChar { it.uppercase() }
        }

        // 2. Restore trailing punctuation mark
        val srcEnd = srcTrimmed.lastOrNull()
        val resEnd = result.lastOrNull()
        val terminalPuncts = listOf('.', '?', '!', '…')

        if (srcEnd != null && terminalPuncts.contains(srcEnd)) {
            if (resEnd == null || !terminalPuncts.contains(resEnd)) {
                result += srcEnd
            } else if (resEnd != srcEnd) {
                result = result.dropLast(1) + srcEnd
            }
        }

        // 3. Collapse 3+ newlines down to at most 2
        while (result.contains("\n\n\n")) {
            result = result.replace("\n\n\n", "\n\n")
        }

        return result
    }

    /**
     * Maps language code to DeepL Target language code (e.g., "en" -> "EN-US").
     * Mirrors `to_deepl_target` in `src/translate.rs:53-86`.
     */
    fun toDeepLTarget(code: String): String? {
        return when (code.lowercase().trim()) {
            "uk" -> "UK"
            "en" -> "EN-US"
            "pl" -> "PL"
            "de" -> "DE"
            "fr" -> "FR"
            "es" -> "ES"
            "it" -> "IT"
            "pt" -> "PT-PT"
            "ru" -> "RU"
            "nl" -> "NL"
            "cs" -> "CS"
            "sk" -> "SK"
            "ro" -> "RO"
            "hu" -> "HU"
            "tr" -> "TR"
            "el" -> "EL"
            "bg" -> "BG"
            "sv" -> "SV"
            "no", "nb" -> "NB"
            "da" -> "DA"
            "fi" -> "FI"
            "lt" -> "LT"
            "lv" -> "LV"
            "et" -> "ET"
            "ja" -> "JA"
            "zh" -> "ZH"
            "ko" -> "KO"
            "ar" -> "AR"
            "id" -> "ID"
            else -> null
        }
    }

    /**
     * Maps language code to DeepL Source language code.
     * Mirrors `to_deepl_source` in `src/translate.rs:88-120`.
     */
    fun toDeepLSource(code: String): String? {
        return when (code.lowercase().trim()) {
            "uk" -> "UK"
            "en" -> "EN"
            "pl" -> "PL"
            "de" -> "DE"
            "fr" -> "FR"
            "es" -> "ES"
            "it" -> "IT"
            "pt" -> "PT"
            "ru" -> "RU"
            "nl" -> "NL"
            "cs" -> "CS"
            "sk" -> "SK"
            "ro" -> "RO"
            "hu" -> "HU"
            "tr" -> "TR"
            "el" -> "EL"
            "bg" -> "BG"
            "sv" -> "SV"
            "no", "nb" -> "NB"
            "da" -> "DA"
            "fi" -> "FI"
            "lt" -> "LT"
            "lv" -> "LV"
            "et" -> "ET"
            "ja" -> "JA"
            "zh" -> "ZH"
            "ko" -> "KO"
            "ar" -> "AR"
            "id" -> "ID"
            else -> null
        }
    }

    /**
     * Checks whether a language pair is supported by DeepL.
     * Mirrors `is_deepl_supported` in `src/translate.rs:351-355`.
     */
    fun isDeepLSupported(srcLang: String, dstLang: String): Boolean {
        val targetOk = toDeepLTarget(dstLang) != null
        val sourceOk = srcLang.equals("auto", ignoreCase = true) || toDeepLSource(srcLang) != null
        return targetOk && sourceOk
    }

    /**
     * Smart heuristic auto-detection:
     * - Russian letters -> Ukrainian
     * - Ukrainian specific letters -> English
     * - General Cyrillic -> English
     * - Others (Latin etc.) -> Ukrainian
     * Mirrors `auto_detect_target_lang` in `src/translate.rs:397-425`.
     */
    fun autoDetectTargetLang(text: String): String {
        var ruSpecific = 0
        var ukSpecific = 0
        var cyrillic = 0
        var latin = 0

        for (c in text) {
            when (c) {
                'ы', 'Ы', 'э', 'Э', 'ъ', 'Ъ', 'ё', 'Ё' -> ruSpecific++
                'і', 'І', 'ї', 'Ї', 'є', 'Є', 'ґ', 'Ґ' -> ukSpecific++
                in '\u0400'..'\u04FF' -> cyrillic++
                in 'a'..'z', in 'A'..'Z' -> latin++
            }
        }

        return when {
            ruSpecific > 0 -> "uk"
            ukSpecific > 0 -> "en"
            cyrillic > latin && cyrillic > 0 -> "en"
            else -> "uk"
        }
    }

    /**
     * Localized language display names (respecting UI language).
     * Mirrors `get_lang_label_for` in `src/translate.rs:460-520`.
     */
    fun getLangLabel(code: String, uiLang: String = "uk"): String {
        val isEn = uiLang.equals("en", ignoreCase = true)
        return when (code.lowercase().trim()) {
            "auto" -> if (isEn) "Auto-detect" else "Автовизначення"
            "uk" -> if (isEn) "Ukrainian" else "Українська"
            "en" -> if (isEn) "English" else "Англійська"
            "pl" -> if (isEn) "Polish" else "Польська"
            "de" -> if (isEn) "German" else "Німецька"
            "fr" -> if (isEn) "French" else "Французька"
            "es" -> if (isEn) "Spanish" else "Іспанська"
            "it" -> if (isEn) "Italian" else "Італійська"
            "pt" -> if (isEn) "Portuguese" else "Португальська"
            "nl" -> if (isEn) "Dutch" else "Нідерландська"
            "cs" -> if (isEn) "Czech" else "Чеська"
            "sk" -> if (isEn) "Slovak" else "Словацька"
            "ro" -> if (isEn) "Romanian" else "Румунська"
            "hu" -> if (isEn) "Hungarian" else "Угорська"
            "tr" -> if (isEn) "Turkish" else "Турецька"
            "el" -> if (isEn) "Greek" else "Грецька"
            "bg" -> if (isEn) "Bulgarian" else "Болгарська"
            "sv" -> if (isEn) "Swedish" else "Шведська"
            "no", "nb" -> if (isEn) "Norwegian" else "Норвезька"
            "da" -> if (isEn) "Danish" else "Данська"
            "fi" -> if (isEn) "Finnish" else "Фінська"
            "lt" -> if (isEn) "Lithuanian" else "Литовська"
            "lv" -> if (isEn) "Latvian" else "Латвійська"
            "et" -> if (isEn) "Estonian" else "Естонська"
            "ja" -> if (isEn) "Japanese" else "Японська"
            "zh" -> if (isEn) "Chinese" else "Китайська"
            "ko" -> if (isEn) "Korean" else "Корейська"
            "ar" -> if (isEn) "Arabic" else "Арабська"
            "he" -> if (isEn) "Hebrew" else "Іврит"
            "hi" -> if (isEn) "Hindi" else "Гінді"
            "ru" -> if (isEn) "Russian" else "Російська"
            else -> code.uppercase()
        }
    }
}
