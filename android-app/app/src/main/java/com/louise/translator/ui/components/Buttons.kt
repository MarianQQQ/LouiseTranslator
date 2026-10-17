package com.louise.translator.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.louise.translator.ui.theme.LouiseTheme

/**
 * TtsPillButton: Identical to `TtsPillButton` in `ui/appwindow.slint:690-735`.
 */
@Composable
fun TtsPillButton(
    isPlaying: Boolean,
    isLoading: Boolean,
    textIdle: String = "Listen",
    textStop: String = "Stop",
    onClick: () -> Unit
) {
    val shape = RoundedCornerShape(4.dp)
    val interactionSource = remember { MutableInteractionSource() }

    Box(
        modifier = Modifier
            .height(20.dp)
            .clip(shape)
            .background(if (isPlaying) Color(0xFF142426) else LouiseTheme.SurfaceCard)
            .border(1.dp, if (isPlaying) LouiseTheme.CyanAccent else LouiseTheme.BorderSubtle, shape)
            .clickable(interactionSource = interactionSource, indication = null, onClick = onClick)
            .padding(horizontal = 6.dp),
        contentAlignment = Alignment.Center
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            val label = when {
                isLoading -> "..."
                isPlaying -> textStop
                else -> textIdle
            }
            val textColor = if (isPlaying) LouiseTheme.CyanAccent else LouiseTheme.TextMuted

            Text(
                text = if (isPlaying) "■" else "🔊",
                fontSize = 9.sp,
                color = textColor
            )
            Text(
                text = label,
                fontSize = 9.5.sp,
                fontFamily = LouiseTheme.MonospaceFont,
                fontWeight = FontWeight.Bold,
                color = textColor
            )
        }
    }
}

/**
 * Compact OCR button for triggering Screen Snip.
 */
@Composable
fun OcrPillButton(
    onClick: () -> Unit
) {
    val shape = RoundedCornerShape(4.dp)
    val interactionSource = remember { MutableInteractionSource() }

    Box(
        modifier = Modifier
            .height(20.dp)
            .clip(shape)
            .background(LouiseTheme.SurfaceCard)
            .border(1.dp, LouiseTheme.BorderSubtle, shape)
            .clickable(interactionSource = interactionSource, indication = null, onClick = onClick)
            .padding(horizontal = 6.dp),
        contentAlignment = Alignment.Center
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            Text(
                text = "⛶",
                fontSize = 9.sp,
                color = LouiseTheme.TextMuted
            )
            Text(
                text = "OCR",
                fontSize = 9.5.sp,
                fontFamily = LouiseTheme.MonospaceFont,
                fontWeight = FontWeight.Bold,
                color = LouiseTheme.TextMuted
            )
        }
    }
}
