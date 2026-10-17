package com.louise.translator

import com.louise.translator.domain.LanguageUtils
import com.louise.translator.domain.TtsUtils
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Characterization & Parity Unit Tests corresponding to Rust tests in `src/main.rs` & `src/tts.rs`.
 */
class LouiseParityTests {

    @Test
    fun testAutoDetectTargetLang() {
        assertEquals("en", LanguageUtils.autoDetectTargetLang("Привіт світ"))
        assertEquals("uk", LanguageUtils.autoDetectTargetLang("Hello world"))
        assertEquals("uk", LanguageUtils.autoDetectTargetLang("Привет мир"))
        assertEquals("en", LanguageUtils.autoDetectTargetLang("Ґрунт і їжа"))
    }

    @Test
    fun testFixTranslationPunctuation() {
        assertEquals("Hello world.", LanguageUtils.fixTranslationPunctuation("Привіт світ.", "hello world"))
        assertEquals("Hello world?", LanguageUtils.fixTranslationPunctuation("Привіт світ?", "hello world."))
        assertEquals("Hello world!", LanguageUtils.fixTranslationPunctuation("Привіт світ!", "hello world"))
    }

    @Test
    fun testDeepLSupportRules() {
        assertTrue(LanguageUtils.isDeepLSupported("auto", "uk"))
        assertTrue(LanguageUtils.isDeepLSupported("en", "de"))
        assertTrue(LanguageUtils.isDeepLSupported("de", "pl"))
        assertFalse(LanguageUtils.isDeepLSupported("en", "non-existent-code"))
    }

    @Test
    fun testNormalizeTextForTts() {
        val raw = "Hello... this is — Louise: translator."
        val normalized = TtsUtils.normalizeTextForTts(raw)
        assertFalse(normalized.contains("..."))
        assertFalse(normalized.contains(" — "))
    }

    @Test
    fun testEdgeVoiceMapping() {
        assertEquals("uk-UA-OstapNeural", TtsUtils.mapLangForEdgeTts("uk").voice)
        assertEquals("en-US-ChristopherNeural", TtsUtils.mapLangForEdgeTts("en").voice)
        assertEquals("de-DE-KillianNeural", TtsUtils.mapLangForEdgeTts("de").voice)
    }
}
