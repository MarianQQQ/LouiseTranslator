package com.louise.translator.data

import com.louise.translator.domain.DeepLUsage
import com.louise.translator.domain.EngineType
import com.louise.translator.domain.LanguageUtils
import com.louise.translator.domain.TranslationResult
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.io.BufferedReader
import java.io.InputStreamReader
import java.io.OutputStreamWriter
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder

/**
 * Clean, lightweight repository for Google Translate & DeepL translation without heavy third-party dependencies.
 * Direct parity with `src/translate.rs:120-395`.
 */
class TranslationRepository {

    /**
     * Translates text with Dual-Engine routing:
     * - DeepL if enabled, key provided, and language pair supported.
     * - Automatic graceful fallback to Google Translate on DeepL errors.
     */
    suspend fun translateText(
        text: String,
        srcLang: String,
        dstLang: String,
        deepLKey: String,
        useDeepL: Boolean
    ): Result<TranslationResult> = withContext(Dispatchers.IO) {
        val trimmed = text.trim()
        if (trimmed.isEmpty()) {
            val label = if (useDeepL && deepLKey.trim().isNotEmpty() && LanguageUtils.isDeepLSupported(srcLang, dstLang)) {
                EngineType.DEEPL
            } else {
                EngineType.GOOGLE
            }
            return@withContext Result.success(
                TranslationResult("", null, label, false)
            )
        }

        if (useDeepL && deepLKey.trim().isNotEmpty()) {
            if (!LanguageUtils.isDeepLSupported(srcLang, dstLang)) {
                return@withContext translateGoogle(trimmed, srcLang, dstLang).map { (trans, detected) ->
                    TranslationResult(trans, detected, EngineType.GOOGLE, fellBack = false)
                }
            }

            try {
                val (trans, detected) = translateDeepL(trimmed, srcLang, dstLang, deepLKey)
                return@withContext Result.success(
                    TranslationResult(trans, detected, EngineType.DEEPL, fellBack = false)
                )
            } catch (e: Exception) {
                // Graceful fallback to Google Translate
                return@withContext translateGoogle(trimmed, srcLang, dstLang).map { (trans, detected) ->
                    TranslationResult(trans, detected, EngineType.GOOGLE, fellBack = true)
                }
            }
        }

        return@withContext translateGoogle(trimmed, srcLang, dstLang).map { (trans, detected) ->
            TranslationResult(trans, detected, EngineType.GOOGLE, fellBack = false)
        }
    }

    /**
     * Translates text via Google Translate single gtx / dict-chrome-ex endpoint.
     */
    suspend fun translateGoogle(
        text: String,
        srcLang: String,
        dstLang: String
    ): Result<Pair<String, String?>> = withContext(Dispatchers.IO) {
        try {
            val encodedQuery = URLEncoder.encode(text, "UTF-8")
            val urlStr = "https://translate.googleapis.com/translate_a/single?client=gtx&sl=$srcLang&tl=$dstLang&dt=t&dj=1"
            val url = URL(urlStr)
            val conn = (url.openConnection() as HttpURLConnection).apply {
                requestMethod = "POST"
                connectTimeout = 5000
                readTimeout = 5000
                doOutput = true
                setRequestProperty("Content-Type", "application/x-www-form-urlencoded")
                setRequestProperty("User-Agent", "Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36")
            }

            conn.outputStream.use { os ->
                OutputStreamWriter(os, "UTF-8").use { writer ->
                    writer.write("q=$encodedQuery")
                    writer.flush()
                }
            }

            if (conn.responseCode !in 200..299) {
                return@withContext Result.failure(Exception("Google Translate HTTP ${conn.responseCode}"))
            }

            val responseText = conn.inputStream.use { stream ->
                BufferedReader(InputStreamReader(stream, "UTF-8")).readText()
            }

            val json = JSONObject(responseText)
            val detectedSrc = if (json.has("src")) json.getString("src") else null
            val resultBuilder = StringBuilder()

            if (json.has("sentences")) {
                val sentences = json.getJSONArray("sentences")
                for (i in 0 until sentences.length()) {
                    val item = sentences.getJSONObject(i)
                    if (item.has("trans")) {
                        resultBuilder.append(item.getString("trans"))
                    }
                }
            }

            val rawResult = resultBuilder.toString()
            val finalTrans = if (rawResult.isEmpty()) text else LanguageUtils.fixTranslationPunctuation(text, rawResult)
            Result.success(Pair(finalTrans, detectedSrc))
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    /**
     * Translates text via official DeepL API v2 endpoint.
     */
    suspend fun translateDeepL(
        text: String,
        srcLang: String,
        dstLang: String,
        apiKey: String
    ): Pair<String, String?> = withContext(Dispatchers.IO) {
        val key = apiKey.trim()
        val endpoint = if (key.endsWith(":fx")) {
            "https://api-free.deepl.com/v2/translate"
        } else {
            "https://api.deepl.com/v2/translate"
        }

        val targetCode = LanguageUtils.toDeepLTarget(dstLang)
            ?: throw IllegalArgumentException("Target language not supported by DeepL: $dstLang")

        val payload = JSONObject().apply {
            put("text", org.json.JSONArray().put(text))
            put("target_lang", targetCode)
            if (!srcLang.equals("auto", ignoreCase = true)) {
                LanguageUtils.toDeepLSource(srcLang)?.let { put("source_lang", it) }
            }
        }

        val url = URL(endpoint)
        val conn = (url.openConnection() as HttpURLConnection).apply {
            requestMethod = "POST"
            connectTimeout = 5000
            readTimeout = 5000
            doOutput = true
            setRequestProperty("Authorization", "DeepL-Auth-Key $key")
            setRequestProperty("Content-Type", "application/json")
        }

        conn.outputStream.use { os ->
            OutputStreamWriter(os, "UTF-8").use { writer ->
                writer.write(payload.toString())
                writer.flush()
            }
        }

        if (conn.responseCode !in 200..299) {
            val err = conn.errorStream?.use { BufferedReader(InputStreamReader(it)).readText() } ?: ""
            throw Exception("DeepL error (${conn.responseCode}): $err")
        }

        val responseText = conn.inputStream.use { BufferedReader(InputStreamReader(it)).readText() }
        val json = JSONObject(responseText)
        val translations = json.getJSONArray("translations")
        if (translations.length() == 0) throw Exception("DeepL returned empty translations")

        val first = translations.getJSONObject(0)
        val translated = first.getString("text")
        val detected = if (first.has("detected_source_language")) first.getString("detected_source_language").lowercase() else null

        Pair(LanguageUtils.fixTranslationPunctuation(text, translated), detected)
    }

    /**
     * Checks DeepL account character usage quota.
     */
    suspend fun fetchDeepLUsage(apiKey: String): Result<DeepLUsage> = withContext(Dispatchers.IO) {
        val key = apiKey.trim()
        if (key.isEmpty()) return@withContext Result.failure(Exception("Empty API key"))

        val endpoint = if (key.endsWith(":fx")) {
            "https://api-free.deepl.com/v2/usage"
        } else {
            "https://api.deepl.com/v2/usage"
        }

        try {
            val url = URL(endpoint)
            val conn = (url.openConnection() as HttpURLConnection).apply {
                requestMethod = "GET"
                connectTimeout = 5000
                readTimeout = 5000
                setRequestProperty("Authorization", "DeepL-Auth-Key $key")
            }

            if (conn.responseCode !in 200..299) {
                return@withContext Result.failure(Exception("HTTP ${conn.responseCode}"))
            }

            val body = conn.inputStream.use { BufferedReader(InputStreamReader(it)).readText() }
            val json = JSONObject(body)
            val count = json.optLong("character_count", 0L)
            val limit = json.optLong("character_limit", 0L)
            Result.success(DeepLUsage(count, limit))
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
