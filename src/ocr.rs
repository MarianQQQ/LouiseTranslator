use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
    SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};

/// Lists all supported OCR language tags installed on this Windows machine.
#[allow(dead_code)]
pub fn get_available_ocr_languages() -> Vec<String> {
    let mut langs = Vec::new();
    if let Ok(vec_langs) = OcrEngine::AvailableRecognizerLanguages() {
        for lang in vec_langs {
            if let Ok(tag) = lang.LanguageTag() {
                langs.push(tag.to_string());
            }
        }
    }
    langs
}

/// Captures a rectangular region of the virtual screen into a top-down BGRA8 pixel buffer.
pub fn capture_screen_region(x: i32, y: i32, width: u32, height: u32) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 {
        return Err("Capture dimensions must be greater than zero".to_string());
    }

    unsafe {
        let display_name: Vec<u16> = "DISPLAY\0".encode_utf16().collect();
        let hdc_screen = windows::Win32::Graphics::Gdi::CreateDCW(
            windows::core::PCWSTR(display_name.as_ptr()),
            windows::core::PCWSTR::null(),
            windows::core::PCWSTR::null(),
            None,
        );
        if hdc_screen.is_invalid() {
            return Err("CreateDCW(DISPLAY) failed".to_string());
        }

        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        if hdc_mem.is_invalid() {
            let _ = DeleteDC(hdc_screen);
            return Err("CreateCompatibleDC failed".to_string());
        }

        let hbm = CreateCompatibleBitmap(hdc_screen, width as i32, height as i32);
        if hbm.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            let _ = DeleteDC(hdc_screen);
            return Err("CreateCompatibleBitmap failed".to_string());
        }

        let old_obj = SelectObject(hdc_mem, hbm.into());

        let blt_res = BitBlt(
            hdc_mem,
            0,
            0,
            width as i32,
            height as i32,
            Some(hdc_screen),
            x,
            y,
            SRCCOPY,
        );

        if let Err(e) = blt_res {
            SelectObject(hdc_mem, old_obj);
            let _ = DeleteObject(hbm.into());
            let _ = DeleteDC(hdc_mem);
            let _ = DeleteDC(hdc_screen);
            return Err(format!("BitBlt failed: {}", e));
        }

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                // Negative height requests a top-down DIB (origin at top-left)
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let buf_len = (width as usize) * (height as usize) * 4;
        let mut pixels = vec![0u8; buf_len];

        let scanlines = GetDIBits(
            hdc_mem,
            hbm,
            0,
            height,
            Some(pixels.as_mut_ptr() as *mut core::ffi::c_void),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        SelectObject(hdc_mem, old_obj);
        let _ = DeleteObject(hbm.into());
        let _ = DeleteDC(hdc_mem);
        let _ = DeleteDC(hdc_screen);

        if scanlines == 0 {
            return Err("GetDIBits returned 0 scanlines".to_string());
        }

        // Ensure alpha channel is 255 for all captured pixels
        for chunk in pixels.chunks_exact_mut(4) {
            chunk[3] = 255;
        }

        Ok(pixels)
    }
}

/// Preprocesses BGRA buffer for higher OCR accuracy on tight or small screen crops:
/// 1. Adds a 16px border filled with the background color (top-left pixel) so characters
///    touching the selection edge aren't clipped by the recognizer.
/// 2. Upscales 2x (bilinear) if the image height is small (< 80px) so tiny UI fonts
///    become crisp and large for Windows Media OCR.
pub fn preprocess_bgra_for_ocr(bgra: &[u8], width: u32, height: u32) -> (Vec<u8>, u32, u32) {
    if width == 0 || height == 0 || bgra.len() < (width as usize) * (height as usize) * 4 {
        return (bgra.to_vec(), width, height);
    }

    // 1. Pad with 16px border using the top-left pixel color
    let pad = 16u32;
    let bg_b = bgra[0];
    let bg_g = bgra[1];
    let bg_r = bgra[2];
    let padded_w = width + pad * 2;
    let padded_h = height + pad * 2;
    let mut padded = vec![0u8; (padded_w as usize) * (padded_h as usize) * 4];

    for px in padded.chunks_exact_mut(4) {
        px[0] = bg_b;
        px[1] = bg_g;
        px[2] = bg_r;
        px[3] = 255;
    }

    for y in 0..height {
        let src_row_start = (y as usize) * (width as usize) * 4;
        let src_row_end = src_row_start + (width as usize) * 4;
        let dst_row_start = (((y + pad) as usize) * (padded_w as usize) + (pad as usize)) * 4;
        let dst_row_end = dst_row_start + (width as usize) * 4;
        padded[dst_row_start..dst_row_end].copy_from_slice(&bgra[src_row_start..src_row_end]);
    }

    // 2. If height is small (< 80px), upscale 2x with bilinear interpolation
    if height < 80 {
        let dst_w = padded_w * 2;
        let dst_h = padded_h * 2;
        let mut upscaled = vec![0u8; (dst_w as usize) * (dst_h as usize) * 4];

        for dy in 0..dst_h {
            let sy_f = (dy as f32) * 0.5;
            let sy0 = (sy_f.floor() as u32).min(padded_h - 1);
            let sy1 = (sy0 + 1).min(padded_h - 1);
            let wy = sy_f - (sy0 as f32);

            for dx in 0..dst_w {
                let sx_f = (dx as f32) * 0.5;
                let sx0 = (sx_f.floor() as u32).min(padded_w - 1);
                let sx1 = (sx0 + 1).min(padded_w - 1);
                let wx = sx_f - (sx0 as f32);

                let idx00 = ((sy0 * padded_w + sx0) as usize) * 4;
                let idx10 = ((sy0 * padded_w + sx1) as usize) * 4;
                let idx01 = ((sy1 * padded_w + sx0) as usize) * 4;
                let idx11 = ((sy1 * padded_w + sx1) as usize) * 4;
                let dst_idx = ((dy * dst_w + dx) as usize) * 4;

                for c in 0..3 {
                    let v00 = padded[idx00 + c] as f32;
                    let v10 = padded[idx10 + c] as f32;
                    let v01 = padded[idx01 + c] as f32;
                    let v11 = padded[idx11 + c] as f32;

                    let top = v00 * (1.0 - wx) + v10 * wx;
                    let bot = v01 * (1.0 - wx) + v11 * wx;
                    let val = top * (1.0 - wy) + bot * wy;
                    upscaled[dst_idx + c] = val.round().clamp(0.0, 255.0) as u8;
                }
                upscaled[dst_idx + 3] = 255;
            }
        }
        return (upscaled, dst_w, dst_h);
    }

    (padded, padded_w, padded_h)
}

/// Maps our translator language code (e.g. "EN", "UK", "DE", "Auto") to a BCP-47 tag prefix.
fn map_translator_lang_to_ocr_tag(lang: &str) -> Option<&'static str> {
    match lang.trim().to_uppercase().as_str() {
        "EN" => Some("en"),
        "UK" => Some("uk"),
        "RU" => Some("ru"),
        "PL" => Some("pl"),
        "DE" => Some("de"),
        "FR" => Some("fr"),
        "ES" => Some("es"),
        "IT" => Some("it"),
        "PT" => Some("pt"),
        "JA" => Some("ja"),
        "ZH" => Some("zh"),
        "KO" => Some("ko"),
        "TR" => Some("tr"),
        "NL" => Some("nl"),
        "CS" => Some("cs"),
        "SK" => Some("sk"),
        "RO" => Some("ro"),
        "HU" => Some("hu"),
        "EL" => Some("el"),
        "BG" => Some("bg"),
        "SV" => Some("sv"),
        "NO" => Some("no"),
        "DA" => Some("da"),
        "FI" => Some("fi"),
        "LT" => Some("lt"),
        "LV" => Some("lv"),
        "ET" => Some("et"),
        "AR" => Some("ar"),
        "HE" => Some("he"),
        "HI" => Some("hi"),
        _ => None,
    }
}

/// Helper to find an installed OCR engine by BCP-47 prefix (e.g. "uk", "ru", "en").
fn try_create_engine_by_prefix(prefix: &str) -> Option<OcrEngine> {
    let prefix_low = prefix.trim().to_lowercase();
    if let Ok(lang) = Language::CreateLanguage(&HSTRING::from(prefix_low.as_str())) {
        if OcrEngine::IsLanguageSupported(&lang).unwrap_or(false) {
            if let Ok(engine) = OcrEngine::TryCreateFromLanguage(&lang) {
                return Some(engine);
            }
        }
    }
    let short_prefix = prefix_low.split('-').next().unwrap_or(&prefix_low);
    if let Ok(vec_langs) = OcrEngine::AvailableRecognizerLanguages() {
        for lang in vec_langs {
            if let Ok(avail_tag) = lang.LanguageTag() {
                let avail_str = avail_tag.to_string().to_lowercase();
                if avail_str == short_prefix || avail_str.starts_with(&format!("{}-", short_prefix)) {
                    if let Ok(engine) = OcrEngine::TryCreateFromLanguage(&lang) {
                        return Some(engine);
                    }
                }
            }
        }
    }
    None
}

/// Creates a Cyrillic-capable OCR engine ("uk" preferred, falling back to "ru" or "bg" if installed).
fn try_create_cyrillic_ocr_engine() -> Option<OcrEngine> {
    try_create_engine_by_prefix("uk")
        .or_else(|| try_create_engine_by_prefix("ru"))
        .or_else(|| try_create_engine_by_prefix("bg"))
}

/// Detects whether output from a Latin-only OCR engine (like `en-US`) looks like misread Cyrillic text.
/// When a Latin OCR model reads Cyrillic words (e.g., "Система розпізнавання тексту"),
/// lowercase Cyrillic letters (`в, н, т, м, я, г, п, и`) are recognized as uppercase Latin letters
/// (`B, H, T, M, R`) or digits (`3, 0, 6`) embedded inside lowercase words ("ChCTeMa p03ni3HaBaHHR TeKCTY").
pub fn looks_like_misread_cyrillic(text: &str) -> bool {
    let mut total_alpha_words = 0usize;
    let mut ransom_words = 0usize;

    for raw_word in text.split_whitespace() {
        let word: String = raw_word
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
        if word.len() < 3 {
            continue;
        }
        let has_alpha = word.chars().any(|c| c.is_ascii_alphabetic());
        if !has_alpha {
            continue;
        }
        total_alpha_words += 1;

        let chars: Vec<char> = word.chars().collect();
        let has_lower = chars.iter().any(|c| c.is_ascii_lowercase());
        let has_upper_after_first = chars.iter().skip(1).any(|c| c.is_ascii_uppercase());
        // Digits mixed inside letters (e.g. "p03ni3HaBaHHR")
        let has_digit_mixed = chars.iter().any(|c| c.is_ascii_digit())
            && chars.iter().filter(|c| c.is_ascii_alphabetic()).count() >= 2;
        // Unusual consonant clusters typical of Cyrillic-as-Latin (e.g. "ChCT", "TeKCT")
        let upper_count = chars.iter().filter(|c| c.is_ascii_uppercase()).count();
        let lower_count = chars.iter().filter(|c| c.is_ascii_lowercase()).count();

        if (has_lower && has_upper_after_first && (upper_count >= 2 || lower_count >= 2))
            || has_digit_mixed
        {
            ransom_words += 1;
        }
    }

    if total_alpha_words == 0 {
        return false;
    }

    // If at least half of words (or >=2 words) exhibit the mixed-case/digit ransom-note pattern
    ransom_words * 2 >= total_alpha_words
}

/// Creates an OcrEngine for a preferred language tag (e.g., "uk", "en-US", "EN"),
/// falling back to English ("en"), UserProfileLanguages, or the first available recognizer language.
pub fn create_ocr_engine(preferred_lang: Option<&str>) -> Result<OcrEngine, String> {
    let resolved_tag = preferred_lang
        .and_then(|l| map_translator_lang_to_ocr_tag(l).or(Some(l.trim())))
        .filter(|l| !l.is_empty() && !l.eq_ignore_ascii_case("auto"));

    if let Some(tag_trimmed) = resolved_tag {
        if let Some(engine) = try_create_engine_by_prefix(tag_trimmed) {
            return Ok(engine);
        }
        // If user requested Ukrainian/Bulgarian/Russian and that exact pack isn't installed,
        // fall back to any installed Cyrillic recognizer before falling back to Latin.
        if matches!(tag_trimmed, "uk" | "ru" | "bg") {
            if let Some(cyr_engine) = try_create_cyrillic_ocr_engine() {
                return Ok(cyr_engine);
            }
        }
    }

    // When preferred_lang is None or "Auto":
    // Prefer English ("en-US" / "en") if available since most screen OCR translations are from English/Latin,
    // or fallback to UserProfileLanguages.
    if let Some(en_engine) = try_create_engine_by_prefix("en") {
        return Ok(en_engine);
    }

    if let Ok(engine) = OcrEngine::TryCreateFromUserProfileLanguages() {
        return Ok(engine);
    }

    if let Ok(vec_langs) = OcrEngine::AvailableRecognizerLanguages() {
        if let Some(first_lang) = vec_langs.into_iter().next() {
            if let Ok(engine) = OcrEngine::TryCreateFromLanguage(&first_lang) {
                return Ok(engine);
            }
        }
    }

    Err("No Windows OCR language packs are installed on this system.".to_string())
}

fn run_engine_on_bitmap(engine: &OcrEngine, bitmap: &SoftwareBitmap) -> Result<String, String> {
    let async_op = engine
        .RecognizeAsync(bitmap)
        .map_err(|e| format!("RecognizeAsync failed: {}", e))?;
    let ocr_result = async_op
        .join()
        .map_err(|e| format!("OCR join failed: {}", e))?;

    let mut lines_out = Vec::new();
    if let Ok(lines) = ocr_result.Lines() {
        for line in lines {
            if let Ok(text) = line.Text() {
                let s = text.to_string();
                if !s.trim().is_empty() {
                    lines_out.push(s);
                }
            }
        }
    }

    if lines_out.is_empty() {
        if let Ok(full_text) = ocr_result.Text() {
            let s = full_text.to_string();
            if !s.trim().is_empty() {
                return Ok(s);
            }
        }
        return Ok(String::new());
    }

    Ok(lines_out.join("\n"))
}

/// Runs Windows Media OCR on raw BGRA8 pixel buffer of size `width` x `height`.
pub fn recognize_bgra_pixels(
    bgra: &[u8],
    width: u32,
    height: u32,
    preferred_lang: Option<&str>,
) -> Result<String, String> {
    if width == 0 || height == 0 {
        return Err("Invalid image dimensions (0x0)".to_string());
    }
    let expected_len = (width as usize) * (height as usize) * 4;
    if bgra.len() < expected_len {
        return Err(format!(
            "Pixel buffer too small: got {}, expected {}",
            bgra.len(),
            expected_len
        ));
    }

    let (prep_bgra, prep_w, prep_h) = preprocess_bgra_for_ocr(&bgra[..expected_len], width, height);

    let writer = DataWriter::new().map_err(|e| format!("DataWriter::new failed: {}", e))?;
    writer
        .WriteBytes(&prep_bgra)
        .map_err(|e| format!("WriteBytes failed: {}", e))?;
    let ibuffer = writer
        .DetachBuffer()
        .map_err(|e| format!("DetachBuffer failed: {}", e))?;

    let bitmap = SoftwareBitmap::CreateCopyWithAlphaFromBuffer(
        &ibuffer,
        BitmapPixelFormat::Bgra8,
        prep_w as i32,
        prep_h as i32,
        BitmapAlphaMode::Ignore,
    )
    .map_err(|e| format!("CreateCopyWithAlphaFromBuffer failed: {}", e))?;

    let is_auto = preferred_lang
        .map(|l| l.trim().is_empty() || l.eq_ignore_ascii_case("auto"))
        .unwrap_or(true);

    let engine = create_ocr_engine(preferred_lang)?;
    let primary_text = run_engine_on_bitmap(&engine, &bitmap)?;

    // Smart Auto-Cyrillic fallback: when in "auto" mode, if the Latin OCR output looks like
    // misread Cyrillic ("ChCTeMa p03ni3HaBaHHR TeKCTY"), re-run with an installed Cyrillic engine.
    if is_auto && looks_like_misread_cyrillic(&primary_text) {
        if let Some(cyr_engine) = try_create_cyrillic_ocr_engine() {
            if let Ok(cyr_text) = run_engine_on_bitmap(&cyr_engine, &bitmap) {
                let has_cyrillic = cyr_text.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
                if !cyr_text.trim().is_empty() && has_cyrillic {
                    return Ok(cyr_text);
                }
            }
        }
    }

    Ok(primary_text)
}

/// Captures a screen rectangle and performs OCR on it in one step.
#[allow(dead_code)]
pub fn capture_and_recognize_rect(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    preferred_lang: Option<&str>,
) -> Result<String, String> {
    let pixels = capture_screen_region(x, y, width, height)?;
    recognize_bgra_pixels(&pixels, width, height, preferred_lang)
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{COLORREF, RECT};
    use windows::Win32::Graphics::Gdi::{
        CreateFontW, CreateSolidBrush, FillRect, GetDC, ReleaseDC, SetBkMode, SetTextColor,
        TextOutW, ANSI_CHARSET, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_PITCH,
        FF_DONTCARE, FW_BOLD, OUT_DEFAULT_PRECIS, TRANSPARENT,
    };

    /// Helper that renders text onto a white GDI bitmap and returns BGRA8 pixels
    fn render_text_to_bgra(text: &str, width: u32, height: u32) -> Vec<u8> {
        unsafe {
            let hdc_screen = GetDC(None);
            let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
            let hbm = CreateCompatibleBitmap(hdc_screen, width as i32, height as i32);
            let old_bm = SelectObject(hdc_mem, hbm.into());

            // Fill white background
            let rect = RECT {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            };
            let white_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
            FillRect(hdc_mem, &rect, white_brush);
            let _ = DeleteObject(white_brush.into());

            // Create clear 28px Arial font
            let font_name: Vec<u16> = "Arial\0".encode_utf16().collect();
            let hfont = CreateFontW(
                28,
                0,
                0,
                0,
                FW_BOLD.0 as i32,
                0,
                0,
                0,
                ANSI_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
                PCWSTR(font_name.as_ptr()),
            );
            let old_font = SelectObject(hdc_mem, hfont.into());

            SetBkMode(hdc_mem, TRANSPARENT);
            SetTextColor(hdc_mem, COLORREF(0x00000000)); // Black text

            let wide_text: Vec<u16> = text.encode_utf16().collect();
            let _ = TextOutW(hdc_mem, 20, 16, &wide_text);

            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width as i32,
                    biHeight: -(height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
            GetDIBits(
                hdc_mem,
                hbm,
                0,
                height,
                Some(pixels.as_mut_ptr() as *mut core::ffi::c_void),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            SelectObject(hdc_mem, old_font);
            let _ = DeleteObject(hfont.into());
            SelectObject(hdc_mem, old_bm);
            let _ = DeleteObject(hbm.into());
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);

            for chunk in pixels.chunks_exact_mut(4) {
                chunk[3] = 255;
            }
            pixels
        }
    }

    #[test]
    fn test_ocr_engine_languages() {
        let langs = get_available_ocr_languages();
        println!("Available OCR languages: {:?}", langs);
        assert!(
            !langs.is_empty(),
            "Windows should have at least one OCR language installed"
        );
        let engine = create_ocr_engine(None);
        assert!(engine.is_ok(), "Should create default OCR engine");
    }

    #[test]
    fn test_ocr_recognizes_rendered_english_text() {
        let pixels = render_text_to_bgra("Louise Translator OCR Test", 420, 64);
        let recognized = recognize_bgra_pixels(&pixels, 420, 64, Some("EN"))
            .expect("OCR recognition should succeed");
        println!("Recognized English text (EN engine): '{}'", recognized);
        assert!(
            recognized.to_lowercase().contains("louise")
                && recognized.to_lowercase().contains("translator"),
            "Expected 'Louise Translator OCR Test', got '{}'",
            recognized
        );

        // Also test what Cyrillic engine outputs on English text and Cyrillic text
        let ru_on_en = recognize_bgra_pixels(&pixels, 420, 64, Some("ru")).unwrap_or_default();
        println!("Recognized English text (RU engine): '{}'", ru_on_en);

        let cyr_pixels = render_text_to_bgra("Система розпізнавання тексту", 480, 64);
        let auto_on_cyr = recognize_bgra_pixels(&cyr_pixels, 480, 64, Some("auto")).unwrap_or_default();
        println!("Recognized Cyrillic text (Auto engine with fallback): '{}'", auto_on_cyr);
        if try_create_cyrillic_ocr_engine().is_some() {
            assert!(
                auto_on_cyr.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)),
                "Expected auto OCR to fallback to Cyrillic engine, got '{}'",
                auto_on_cyr
            );
        }
    }

    #[test]
    fn test_looks_like_misread_cyrillic() {
        assert!(looks_like_misread_cyrillic("ChCTeMa p03ni3HaBaHHR TeKCTY"));
        assert!(looks_like_misread_cyrillic("npBiT CBiT"));
        assert!(!looks_like_misread_cyrillic("Louise Translator OCR Test"));
        assert!(!looks_like_misread_cyrillic("Hello world, this is normal English text!"));
    }

    #[test]
    fn test_preprocess_bgra_for_ocr() {
        // Small image (40x20) should be padded (+32 -> 72x52) and upscaled 2x (-> 144x104)
        let small = vec![255u8; 40 * 20 * 4];
        let (out, w, h) = preprocess_bgra_for_ocr(&small, 40, 20);
        assert_eq!(w, (40 + 32) * 2);
        assert_eq!(h, (20 + 32) * 2);
        assert_eq!(out.len(), (w * h * 4) as usize);

        // Large image (200x100) should only be padded (+32 -> 232x132), not upscaled
        let large = vec![255u8; 200 * 100 * 4];
        let (out2, w2, h2) = preprocess_bgra_for_ocr(&large, 200, 100);
        assert_eq!(w2, 200 + 32);
        assert_eq!(h2, 100 + 32);
        assert_eq!(out2.len(), (w2 * h2 * 4) as usize);
    }
}
