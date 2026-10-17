package com.louise.translator.domain

/**
 * Text normalization and Edge TTS voice mappings corresponding to `src/tts.rs`.
 */
object TtsUtils {

    data class EdgeVoice(
        val voice: String,
        val locale: String
    )

    /**
     * Maps language code to Google TTS language code.
     * Mirrors `map_lang_for_tts` in `src/tts.rs:20-54`.
     */
    fun mapLangForGoogleTts(lang: String): String {
        return when (lang.trim().lowercase()) {
            "uk" -> "uk"
            "en" -> "en"
            "pl" -> "pl"
            "de" -> "de"
            "fr" -> "fr"
            "es" -> "es"
            "it" -> "it"
            "pt" -> "pt"
            "nl" -> "nl"
            "cs" -> "cs"
            "sk" -> "sk"
            "ro" -> "ro"
            "hu" -> "hu"
            "tr" -> "tr"
            "el" -> "el"
            "bg" -> "bg"
            "sv" -> "sv"
            "no" -> "no"
            "da" -> "da"
            "fi" -> "fi"
            "lt" -> "lt"
            "lv" -> "lv"
            "et" -> "et"
            "ja" -> "ja"
            "zh" -> "zh-CN"
            "ko" -> "ko"
            "ar" -> "ar"
            "he" -> "iw" // Google TTS uses "iw" for Hebrew
            "hi" -> "hi"
            "ru" -> "ru"
            else -> "en"
        }
    }

    /**
     * Normalizes text before TTS synthesis to eliminate unnatural pauses while maintaining natural flow.
     * Mirrors `normalize_text_for_tts` in `src/tts.rs:58-99`.
     */
    fun normalizeTextForTts(text: String): String {
        val lines = text.lines().map { it.trim() }.filter { it.isNotEmpty() }
        val total = lines.size
        val mergedLines = mutableListOf<String>()

        for ((idx, trimmed) in lines.withIndex()) {
            var lineStr = trimmed
            if (idx + 1 < total) {
                val lastCh = lineStr.lastOrNull()
                if (lastCh != null && lastCh !in listOf('.', '!', '?', ',', ';', ':')) {
                    lineStr += ","
                }
            }
            mergedLines.add(lineStr)
        }

        var s = mergedLines.joinToString(" ")
        s = s.replace("...", ",")
            .replace("…", ",")
            .replace(" — ", ", ")
            .replace(" – ", ", ")
            .replace(" - ", ", ")
            .replace("; ", ", ")
            .replace(": ", ", ")

        while (s.contains(". ")) {
            s = s.replace(". ", ", ")
        }
        while (s.contains(",,")) {
            s = s.replace(",,", ",")
        }

        return s.split("\\s+".toRegex()).joinToString(" ")
    }

    /**
     * Maps language code to the best Microsoft Edge neural voice.
     * Mirrors `map_lang_for_edge_tts` in `src/tts.rs:163-255`.
     */
    fun mapLangForEdgeTts(lang: String): EdgeVoice {
        return when (lang.trim().lowercase()) {
            "uk" -> EdgeVoice("uk-UA-OstapNeural", "uk-UA")
            "en" -> EdgeVoice("en-US-ChristopherNeural", "en-US")
            "pl" -> EdgeVoice("pl-PL-MarekNeural", "pl-PL")
            "de" -> EdgeVoice("de-DE-KillianNeural", "de-DE")
            "fr" -> EdgeVoice("fr-FR-HenriNeural", "fr-FR")
            "es" -> EdgeVoice("es-ES-AlvaroNeural", "es-ES")
            "it" -> EdgeVoice("it-IT-DiegoNeural", "it-IT")
            "pt" -> EdgeVoice("pt-BR-AntonioNeural", "pt-BR")
            "nl" -> EdgeVoice("nl-NL-MaartenNeural", "nl-NL")
            "cs" -> EdgeVoice("cs-CZ-AntoninNeural", "cs-CZ")
            "sk" -> EdgeVoice("sk-SK-LukasNeural", "sk-SK")
            "ro" -> EdgeVoice("ro-RO-EmilNeural", "ro-RO")
            "hu" -> EdgeVoice("hu-HU-TamasNeural", "hu-HU")
            "tr" -> EdgeVoice("tr-TR-AhmetNeural", "tr-TR")
            "el" -> EdgeVoice("el-GR-NestorasNeural", "el-GR")
            "bg" -> EdgeVoice("bg-BG-BorislavNeural", "bg-BG")
            "sv" -> EdgeVoice("sv-SE-MattiasNeural", "sv-SE")
            "no" -> EdgeVoice("nb-NO-FinnNeural", "nb-NO")
            "da" -> EdgeVoice("da-DK-JeppeNeural", "da-DK")
            "fi" -> EdgeVoice("fi-FI-HarriNeural", "fi-FI")
            "lt" -> EdgeVoice("lt-LT-LeonasNeural", "lt-LT")
            "lv" -> EdgeVoice("lv-LV-NilsNeural", "lv-LV")
            "et" -> EdgeVoice("et-EE-KertNeural", "et-EE")
            "ja" -> EdgeVoice("ja-JP-KeitaNeural", "ja-JP")
            "zh" -> EdgeVoice("zh-CN-YunxiNeural", "zh-CN")
            "ko" -> EdgeVoice("ko-KR-InJoonNeural", "ko-KR")
            "ar" -> EdgeVoice("ar-SA-HamedNeural", "ar-SA")
            "he" -> EdgeVoice("he-IL-AvriNeural", "he-IL")
            "hi" -> EdgeVoice("hi-IN-MadhurNeural", "hi-IN")
            "ru" -> EdgeVoice("ru-RU-DmitryNeural", "ru-RU")
            else -> EdgeVoice("en-US-ChristopherNeural", "en-US")
        }
    }
}
