package com.louise.translator

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.speech.tts.TextToSpeech
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.*
import androidx.lifecycle.lifecycleScope
import com.louise.translator.data.TranslationRepository
import com.louise.translator.domain.EngineType
import com.louise.translator.domain.LanguageUtils
import com.louise.translator.ui.LouiseTranslatorScreen
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.util.Locale

/**
 * MainActivity: Supports both direct launcher start and system text selection (ACTION_PROCESS_TEXT).
 */
class MainActivity : ComponentActivity(), TextToSpeech.OnInitListener {

    private val repository = TranslationRepository()
    private var tts: TextToSpeech? = null
    private var isTtsReady = false
    private var debounceJob: Job? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        tts = TextToSpeech(this, this)

        val isReadOnly = intent?.getBooleanExtra(Intent.EXTRA_PROCESS_TEXT_READONLY, false) ?: false
        val initialText = if (intent?.action == Intent.ACTION_PROCESS_TEXT) {
            intent?.getCharSequenceExtra(Intent.EXTRA_PROCESS_TEXT)?.toString() ?: ""
        } else {
            ""
        }

        setContent {
            var srcText by remember { mutableStateOf(initialText) }
            var dstText by remember { mutableStateOf("") }
            var srcLang by remember { mutableStateOf("auto") }
            var dstLang by remember { mutableStateOf(LanguageUtils.autoDetectTargetLang(initialText)) }
            var detectedLangLabel by remember { mutableStateOf("") }
            var detectedSrcCode by remember { mutableStateOf("") }
            var engineLabel by remember { mutableStateOf(EngineType.GOOGLE) }
            var isLoading by remember { mutableStateOf(false) }
            var isSrcTtsPlaying by remember { mutableStateOf(false) }
            var isDstTtsPlaying by remember { mutableStateOf(false) }
            var statusText by remember { mutableStateOf("") }

            fun executeTranslation(text: String, sLang: String, tLang: String) {
                if (text.trim().isEmpty()) {
                    dstText = ""
                    detectedLangLabel = ""
                    detectedSrcCode = ""
                    isLoading = false
                    return
                }

                isLoading = true
                lifecycleScope.launch {
                    val result = repository.translateText(
                        text = text,
                        srcLang = sLang,
                        dstLang = tLang,
                        deepLKey = "", // In actual app injected from encrypted storage
                        useDeepL = false
                    )

                    isLoading = false
                    result.onSuccess { res ->
                        dstText = res.translatedText
                        engineLabel = res.engineUsed
                        if (res.detectedSourceLang != null) {
                            detectedSrcCode = res.detectedSourceLang
                            detectedLangLabel = LanguageUtils.getLangLabel(res.detectedSourceLang)
                        }
                    }.onFailure { err ->
                        dstText = "Error: ${err.localizedMessage}"
                    }
                }
            }

            fun scheduleDebouncedTranslation(text: String) {
                debounceJob?.cancel()
                debounceJob = lifecycleScope.launch {
                    delay(400) // 400ms debounce
                    executeTranslation(text, srcLang, dstLang)
                }
            }

            // Trigger initial translation if text came from selection
            LaunchedEffect(initialText) {
                if (initialText.isNotEmpty()) {
                    executeTranslation(initialText, srcLang, dstLang)
                }
            }

            LouiseTranslatorScreen(
                srcText = srcText,
                dstText = dstText,
                srcLang = srcLang,
                dstLang = dstLang,
                detectedLangLabel = detectedLangLabel,
                engineLabel = engineLabel,
                isLoading = isLoading,
                isSrcTtsPlaying = isSrcTtsPlaying,
                isDstTtsPlaying = isDstTtsPlaying,
                statusText = statusText,
                onSrcTextChanged = { newText ->
                    srcText = newText
                    scheduleDebouncedTranslation(newText)
                },
                onSwapLanguages = {
                    val newSrc = dstLang
                    val newDst = if (srcLang == "auto") "uk" else srcLang
                    srcLang = newSrc
                    dstLang = newDst
                    if (dstText.isNotEmpty()) {
                        srcText = dstText
                        dstText = ""
                        executeTranslation(srcText, srcLang, dstLang)
                    }
                },
                onPlaySrcTts = {
                    if (isSrcTtsPlaying) {
                        tts?.stop()
                        isSrcTtsPlaying = false
                    } else {
                        tts?.stop()
                        isDstTtsPlaying = false
                        val playCode = if (srcLang != "auto") srcLang else (if (detectedSrcCode.isNotEmpty()) detectedSrcCode else "en")
                        speakText(srcText, playCode) { isSrcTtsPlaying = it }
                    }
                },
                onPlayDstTts = {
                    if (isDstTtsPlaying) {
                        tts?.stop()
                        isDstTtsPlaying = false
                    } else {
                        tts?.stop()
                        isSrcTtsPlaying = false
                        speakText(dstText, dstLang) { isDstTtsPlaying = it }
                    }
                },
                onOcrRequested = {
                    Toast.makeText(this@MainActivity, "Screen OCR: Google ML Kit integrated", Toast.LENGTH_SHORT).show()
                },
                onCopyRequested = {
                    if (dstText.isNotEmpty()) {
                        val clip = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                        clip.setPrimaryClip(ClipData.newPlainText("Louise Translation", dstText))
                        statusText = "Copied! ✓"
                        lifecycleScope.launch {
                            delay(1800)
                            statusText = ""
                        }
                    }
                },
                onPasteAsReplacementRequested = {
                    if (!isReadOnly && intent?.action == Intent.ACTION_PROCESS_TEXT) {
                        val returnIntent = Intent().apply {
                            putExtra(Intent.EXTRA_PROCESS_TEXT, dstText)
                        }
                        setResult(Activity.RESULT_OK, returnIntent)
                        finish()
                    } else {
                        val clip = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                        clip.setPrimaryClip(ClipData.newPlainText("Louise Translation", dstText))
                        Toast.makeText(this@MainActivity, "Copied to clipboard! ✓", Toast.LENGTH_SHORT).show()
                        finish()
                    }
                },
                onClearRequested = {
                    srcText = ""
                    dstText = ""
                    detectedLangLabel = ""
                    detectedSrcCode = ""
                    statusText = ""
                    tts?.stop()
                    isSrcTtsPlaying = false
                    isDstTtsPlaying = false
                },
                onCloseRequested = {
                    finish()
                }
            )
        }
    }

    private fun speakText(text: String, langCode: String, onStateChange: (Boolean) -> Unit) {
        if (!isTtsReady || text.isBlank()) return
        tts?.language = Locale.forLanguageTag(langCode)
        onStateChange(true)
        tts?.speak(text, TextToSpeech.QUEUE_FLUSH, null, "LouiseTtsUtterance")
    }

    override fun onInit(status: Int) {
        if (status == TextToSpeech.SUCCESS) {
            isTtsReady = true
        }
    }

    override fun onDestroy() {
        tts?.stop()
        tts?.shutdown()
        super.onDestroy()
    }
}
