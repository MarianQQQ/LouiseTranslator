package com.louise.translator.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.louise.translator.domain.EngineType
import com.louise.translator.domain.LanguageUtils
import com.louise.translator.ui.components.OcrPillButton
import com.louise.translator.ui.components.TtsPillButton
import com.louise.translator.ui.theme.LouiseTheme

/**
 * Authentic Louise Translator Screen Replica matching `ui/appwindow.slint`.
 */
@Composable
fun LouiseTranslatorScreen(
    srcText: String,
    dstText: String,
    srcLang: String,
    dstLang: String,
    detectedLangLabel: String,
    engineLabel: EngineType,
    isLoading: Boolean,
    isSrcTtsPlaying: Boolean,
    isDstTtsPlaying: Boolean,
    statusText: String,
    onSrcTextChanged: (String) -> Unit,
    onSwapLanguages: () -> Unit,
    onPlaySrcTts: () -> Unit,
    onPlayDstTts: () -> Unit,
    onOcrRequested: () -> Unit,
    onCopyRequested: () -> Unit,
    onPasteAsReplacementRequested: () -> Unit,
    onClearRequested: () -> Unit,
    onCloseRequested: () -> Unit
) {
    val cardShape = RoundedCornerShape(8.dp)

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp))
            .background(LouiseTheme.Background)
            .padding(14.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        // 1. Titlebar Header
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Text(text = "🌐", fontSize = 14.sp)
                Text(
                    text = "Louise Translator",
                    color = LouiseTheme.TextPrimary,
                    fontFamily = LouiseTheme.MonospaceFont,
                    fontWeight = FontWeight.Bold,
                    fontSize = 13.sp
                )
            }

            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                Text(
                    text = engineLabel.label,
                    color = if (engineLabel == EngineType.DEEPL) LouiseTheme.DeepLBlue else LouiseTheme.TextMuted,
                    fontFamily = LouiseTheme.MonospaceFont,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    text = "✕",
                    color = LouiseTheme.TextMuted,
                    fontSize = 13.sp,
                    modifier = Modifier.clickable { onCloseRequested() }
                )
            }
        }

        // 2. Language Selectors Bar (Source ⇄ Target)
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .weight(1f)
                    .height(34.dp)
                    .background(LouiseTheme.SurfaceCard, cardShape)
                    .border(1.dp, LouiseTheme.BorderSubtle, cardShape)
                    .padding(horizontal = 10.dp),
                contentAlignment = Alignment.CenterStart
            ) {
                Text(
                    text = LanguageUtils.getLangLabel(srcLang),
                    color = LouiseTheme.TextPrimary,
                    fontFamily = LouiseTheme.MonospaceFont,
                    fontSize = 11.5.sp
                )
            }

            Box(
                modifier = Modifier
                    .size(34.dp)
                    .clip(RoundedCornerShape(6.dp))
                    .background(LouiseTheme.SurfaceCard)
                    .border(1.dp, LouiseTheme.BorderSubtle, RoundedCornerShape(6.dp))
                    .clickable { onSwapLanguages() },
                contentAlignment = Alignment.Center
            ) {
                Text(text = "⇄", color = LouiseTheme.TextMuted, fontSize = 14.sp)
            }

            Box(
                modifier = Modifier
                    .weight(1f)
                    .height(34.dp)
                    .background(LouiseTheme.SurfaceCard, cardShape)
                    .border(1.dp, LouiseTheme.BorderSubtle, cardShape)
                    .padding(horizontal = 10.dp),
                contentAlignment = Alignment.CenterStart
            ) {
                Text(
                    text = LanguageUtils.getLangLabel(dstLang),
                    color = LouiseTheme.TextPrimary,
                    fontFamily = LouiseTheme.MonospaceFont,
                    fontSize = 11.5.sp
                )
            }
        }

        // 3. SOURCE Box Header & TextInput
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    Text(
                        text = "SOURCE",
                        color = LouiseTheme.TextMuted,
                        fontSize = 10.sp,
                        fontFamily = LouiseTheme.MonospaceFont,
                        fontWeight = FontWeight.Bold
                    )
                    if (detectedLangLabel.isNotEmpty()) {
                        Text(
                            text = "→ $detectedLangLabel",
                            color = LouiseTheme.TextDim,
                            fontSize = 10.sp,
                            fontFamily = LouiseTheme.MonospaceFont
                        )
                    }
                }

                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    if (srcText.isNotEmpty()) {
                        Text(
                            text = "${srcText.length} chars",
                            color = LouiseTheme.TextDim,
                            fontSize = 10.sp,
                            fontFamily = LouiseTheme.MonospaceFont
                        )
                    }
                    OcrPillButton(onClick = onOcrRequested)
                    TtsPillButton(
                        isPlaying = isSrcTtsPlaying,
                        isLoading = false,
                        onClick = onPlaySrcTts
                    )
                }
            }

            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(115.dp)
                    .background(LouiseTheme.SurfaceCard, cardShape)
                    .border(1.dp, LouiseTheme.BorderSubtle, cardShape)
                    .padding(10.dp)
            ) {
                BasicTextField(
                    value = srcText,
                    onValueChange = onSrcTextChanged,
                    modifier = Modifier.fillMaxSize(),
                    textStyle = TextStyle(
                        color = LouiseTheme.TextPrimary,
                        fontFamily = LouiseTheme.MonospaceFont,
                        fontSize = 12.5.sp
                    ),
                    cursorBrush = SolidColor(LouiseTheme.TextPrimary)
                )
                if (srcText.isEmpty()) {
                    Text(
                        text = "Type text or select on screen...",
                        color = LouiseTheme.TextDim,
                        fontFamily = LouiseTheme.MonospaceFont,
                        fontSize = 12.sp
                    )
                }
            }
        }

        // 4. TRANSLATION Box Header & Output
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = if (isLoading) "TRANSLATING..." else "TRANSLATION",
                    color = if (isLoading) LouiseTheme.TextPrimary else LouiseTheme.TextMuted,
                    fontSize = 10.sp,
                    fontFamily = LouiseTheme.MonospaceFont,
                    fontWeight = FontWeight.Bold
                )

                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    if (statusText.isNotEmpty()) {
                        Text(
                            text = statusText,
                            color = Color(0xFF88FF88),
                            fontSize = 10.sp,
                            fontFamily = LouiseTheme.MonospaceFont,
                            fontWeight = FontWeight.Bold
                        )
                    }
                    TtsPillButton(
                        isPlaying = isDstTtsPlaying,
                        isLoading = false,
                        onClick = onPlayDstTts
                    )
                }
            }

            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(115.dp)
                    .background(LouiseTheme.SurfaceCard, cardShape)
                    .border(1.dp, LouiseTheme.BorderSubtle, cardShape)
                    .padding(10.dp)
            ) {
                Text(
                    text = dstText,
                    color = LouiseTheme.TextPrimary,
                    fontFamily = LouiseTheme.MonospaceFont,
                    fontSize = 12.5.sp
                )
            }
        }

        // 5. Action Buttons (Copy | Paste as replacement | Clear)
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Button(
                onClick = onCopyRequested,
                modifier = Modifier.weight(1f).height(38.dp),
                shape = cardShape,
                colors = ButtonDefaults.buttonColors(containerColor = LouiseTheme.ButtonSecNormal)
            ) {
                Text("Copy", color = LouiseTheme.TextPrimary, fontFamily = LouiseTheme.MonospaceFont, fontSize = 11.5.sp)
            }

            Button(
                onClick = onPasteAsReplacementRequested,
                modifier = Modifier.weight(2f).height(38.dp),
                shape = cardShape,
                colors = ButtonDefaults.buttonColors(containerColor = LouiseTheme.ButtonPriNormal)
            ) {
                Text("Paste as replacement", color = LouiseTheme.ButtonPriText, fontFamily = LouiseTheme.MonospaceFont, fontWeight = FontWeight.Bold, fontSize = 11.5.sp)
            }

            Button(
                onClick = onClearRequested,
                modifier = Modifier.weight(1f).height(38.dp),
                shape = cardShape,
                colors = ButtonDefaults.buttonColors(containerColor = LouiseTheme.ButtonSecNormal)
            ) {
                Text("Clear", color = LouiseTheme.TextPrimary, fontFamily = LouiseTheme.MonospaceFont, fontSize = 11.5.sp)
            }
        }
    }
}
