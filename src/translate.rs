use std::time::Duration;
use crate::AppWindow;

// Post-processing of translation: restores punctuation marks and initial letter casing
// which translation APIs occasionally drop for certain language pairs
pub fn fix_translation_punctuation(src: &str, translated: &str) -> String {
    if translated.is_empty() {
        return translated.to_string();
    }

    let src_trimmed = src.trim();
    let mut result = translated.trim().to_string();

    // 1. Restore capitalized initial letter (if original begins with uppercase)
    if let Some(first_src_char) = src_trimmed.chars().next() {
        if first_src_char.is_uppercase() {
            // Capitalize first character of translation
            let mut chars = result.chars();
            if let Some(first) = chars.next() {
                let upper: String = first.to_uppercase().collect();
                result = upper + chars.as_str();
            }
        }
    }

    // 2. Restore trailing punctuation mark
    // Take the last significant character of original text
    let src_end = src_trimmed.chars().last();
    let res_end = result.chars().last();

    let terminal_puncts = ['.', '?', '!', '…'];
    if let Some(sp) = src_end {
        if terminal_puncts.contains(&sp) {
            // If translation does not end with the same or any terminal punctuation
            if !res_end.map(|c| terminal_puncts.contains(&c)).unwrap_or(false) {
                result.push(sp);
            } else if res_end != Some(sp) {
                // Replace terminal punctuation mark with matching one (e.g. . -> ?)
                result.pop();
                result.push(sp);
            }
        }
    }

    // 3. Collapse 3+ newlines down to at most 2 (prevent excessive paragraph spacing)
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }

    result
}

pub fn to_deepl_target(code: &str) -> Option<&'static str> {
    match code.to_lowercase().as_str() {
        "uk" => Some("UK"),
        "en" => Some("EN-US"),
        "pl" => Some("PL"),
        "de" => Some("DE"),
        "fr" => Some("FR"),
        "es" => Some("ES"),
        "it" => Some("IT"),
        "pt" => Some("PT-PT"),
        "ru" => Some("RU"),
        "nl" => Some("NL"),
        "cs" => Some("CS"),
        "sk" => Some("SK"),
        "ro" => Some("RO"),
        "hu" => Some("HU"),
        "tr" => Some("TR"),
        "el" => Some("EL"),
        "bg" => Some("BG"),
        "sv" => Some("SV"),
        "no" | "nb" => Some("NB"),
        "da" => Some("DA"),
        "fi" => Some("FI"),
        "lt" => Some("LT"),
        "lv" => Some("LV"),
        "et" => Some("ET"),
        "ja" => Some("JA"),
        "zh" => Some("ZH"),
        "ko" => Some("KO"),
        "ar" => Some("AR"),
        "id" => Some("ID"),
        _ => None,
    }
}

pub fn to_deepl_source(code: &str) -> Option<&'static str> {
    match code.to_lowercase().as_str() {
        "uk" => Some("UK"),
        "en" => Some("EN"),
        "pl" => Some("PL"),
        "de" => Some("DE"),
        "fr" => Some("FR"),
        "es" => Some("ES"),
        "it" => Some("IT"),
        "pt" => Some("PT"),
        "ru" => Some("RU"),
        "nl" => Some("NL"),
        "cs" => Some("CS"),
        "sk" => Some("SK"),
        "ro" => Some("RO"),
        "hu" => Some("HU"),
        "tr" => Some("TR"),
        "el" => Some("EL"),
        "bg" => Some("BG"),
        "sv" => Some("SV"),
        "no" | "nb" => Some("NB"),
        "da" => Some("DA"),
        "fi" => Some("FI"),
        "lt" => Some("LT"),
        "lv" => Some("LV"),
        "et" => Some("ET"),
        "ja" => Some("JA"),
        "zh" => Some("ZH"),
        "ko" => Some("KO"),
        "ar" => Some("AR"),
        "id" => Some("ID"),
        _ => None,
    }
}

pub async fn translate_deepl(
    text: &str,
    src_lang: &str,
    dst_lang: &str,
    api_key: &str,
) -> Result<(String, Option<String>), String> {
    let target = to_deepl_target(dst_lang).ok_or_else(|| "Target language not supported by DeepL".to_string())?;
    let endpoint = if api_key.trim().ends_with(":fx") {
        "https://api-free.deepl.com/v2/translate"
    } else {
        "https://api.deepl.com/v2/translate"
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let mut body = serde_json::json!({
        "text": [text],
        "target_lang": target,
    });

    if src_lang != "auto" {
        if let Some(src) = to_deepl_source(src_lang) {
            body["source_lang"] = serde_json::json!(src);
        }
    }

    let res = client
        .post(endpoint)
        .header("Authorization", format!("DeepL-Auth-Key {}", api_key.trim()))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status = res.status();
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("DeepL API error ({}): {}", status, err_text));
    }

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let trans = json.get("translations")
        .and_then(|t| t.as_array())
        .and_then(|a| a.get(0))
        .ok_or_else(|| "Invalid DeepL response format".to_string())?;

    let translated_text = trans.get("text")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let detected = trans.get("detected_source_language")
        .and_then(|s| s.as_str())
        .map(|s| s.to_lowercase());

    if translated_text.is_empty() {
        Err("DeepL returned empty text".to_string())
    } else {
        Ok((fix_translation_punctuation(text, &translated_text), detected))
    }
}

pub fn format_num_spaces(mut n: u64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut parts = Vec::new();
    while n > 0 {
        let rem = n % 1000;
        n /= 1000;
        if n > 0 {
            parts.push(format!("{:03}", rem));
        } else {
            parts.push(format!("{}", rem));
        }
    }
    parts.reverse();
    parts.join(" ")
}

pub async fn fetch_deepl_usage(api_key: &str) -> Result<(u64, u64), String> {
    if api_key.trim().is_empty() {
        return Err("No key".to_string());
    }
    let endpoint = if api_key.trim().ends_with(":fx") {
        "https://api-free.deepl.com/v2/usage"
    } else {
        "https://api.deepl.com/v2/usage"
    };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client
        .get(endpoint)
        .header("Authorization", format!("DeepL-Auth-Key {}", api_key.trim()))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status = res.status();
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Error {}: {}", status, err_text));
    }

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let count = json.get("character_count").and_then(|v| v.as_u64()).unwrap_or(0);
    let limit = json.get("character_limit").and_then(|v| v.as_u64()).unwrap_or(0);
    Ok((count, limit))
}

pub fn update_deepl_quota_display(app_weak: slint::Weak<AppWindow>, api_key: String) {
    if api_key.trim().is_empty() {
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_deepl_quota_text("".into());
                app.set_deepl_quota_percent(1.0);
            }
        });
        return;
    }
    let app_w_load = app_weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(app) = app_w_load.upgrade() {
            app.set_deepl_quota_loading(true);
        }
    });

    tokio::spawn(async move {
        let res = fetch_deepl_usage(&api_key).await;
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_deepl_quota_loading(false);
                match res {
                    Ok((count, limit)) => {
                        let remaining = limit.saturating_sub(count);
                        let frac = if limit > 0 {
                            (remaining as f32) / (limit as f32)
                        } else {
                            0.0
                        };
                        let percent = frac * 100.0;
                        let text = format!("{} / {} ({:.1}%)", format_num_spaces(remaining), format_num_spaces(limit), percent);
                        app.set_deepl_quota_text(text.into());
                        app.set_deepl_quota_percent(frac);
                    }
                    Err(_) => {
                        let err_msg = if app.get_ui_lang() == "en" { "Unavailable" } else { "Недоступно" };
                        app.set_deepl_quota_text(err_msg.into());
                    }
                }
            }
        });
    });
}

pub async fn translate_google(
    text: &str,
    src_lang: &str,
    dst_lang: &str,
) -> Result<(String, Option<String>), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=dict-chrome-ex&sl={}&tl={}&dt=t&dj=1",
        src_lang, dst_lang
    );

    let form = [("q", text)];
    let response = client
        .post(&url)
        .form(&form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

    let mut result = String::new();
    let mut detected_src: Option<String> = json.get("src").and_then(|s| s.as_str()).map(|s| s.to_string());

    if let Some(sentences) = json.get("sentences").and_then(|s| s.as_array()) {
        for s in sentences {
            if let Some(trans) = s.get("trans").and_then(|t| t.as_str()) {
                result.push_str(trans);
            }
        }
    }

    if result.is_empty() {
        let url_fallback = format!(
            "https://translate.googleapis.com/translate_a/single?client=gtx&sl={}&tl={}&dt=t&dj=1",
            src_lang, dst_lang
        );
        if let Ok(resp2) = client.post(&url_fallback).form(&form).send().await {
            if let Ok(json2) = resp2.json::<serde_json::Value>().await {
                if detected_src.is_none() {
                    detected_src = json2.get("src").and_then(|s| s.as_str()).map(|s| s.to_string());
                }
                if let Some(sentences) = json2.get("sentences").and_then(|s| s.as_array()) {
                    for s in sentences {
                        if let Some(trans) = s.get("trans").and_then(|t| t.as_str()) {
                            result.push_str(trans);
                        }
                    }
                }
            }
        }
    }

    if result.is_empty() {
        Ok((text.to_string(), detected_src))
    } else {
        Ok((fix_translation_punctuation(text, &result), detected_src))
    }
}

pub async fn translate_text(
    text: &str,
    src_lang: &str,
    dst_lang: &str,
    deepl_key: &str,
    use_deepl: bool,
) -> Result<(String, Option<String>, String, bool), String> {
    if text.trim().is_empty() {
        let label = if use_deepl && !deepl_key.trim().is_empty() {
            "DeepL ⚡"
        } else {
            "Google"
        };
        return Ok((String::new(), None, label.to_string(), false));
    }

    if use_deepl && !deepl_key.trim().is_empty() {
        match translate_deepl(text, src_lang, dst_lang, deepl_key).await {
            Ok((trans, detected)) => {
                return Ok((trans, detected, "DeepL ⚡".to_string(), false));
            }
            Err(e) => {
                eprintln!("[Louise] DeepL failed ({}), falling back to Google Translate", e);
                let (trans, detected) = translate_google(text, src_lang, dst_lang).await?;
                return Ok((trans, detected, "Google 🔄".to_string(), true));
            }
        }
    }

    let (trans, detected) = translate_google(text, src_lang, dst_lang).await?;
    Ok((trans, detected, "Google".to_string(), false))
}

// Auto-detection: Russian letters -> Ukrainian, Ukrainian letters -> English, general Cyrillic -> English, others -> Ukrainian
pub fn auto_detect_target_lang(text: &str) -> &'static str {
    let mut ru_specific = 0;
    let mut uk_specific = 0;
    let mut cyrillic = 0;
    let mut latin = 0;

    for c in text.chars() {
        let lower = c.to_ascii_lowercase();
        if c == 'ы' || c == 'Ы' || c == 'э' || c == 'Э' || c == 'ъ' || c == 'Ъ' || c == 'ё' || c == 'Ё' {
            ru_specific += 1;
        } else if c == 'і' || c == 'І' || c == 'ї' || c == 'Ї' || c == 'є' || c == 'Є' || c == 'ґ' || c == 'Ґ' {
            uk_specific += 1;
        } else if ('\u{0400}'..='\u{04FF}').contains(&c) {
            cyrillic += 1;
        } else if lower.is_ascii_alphabetic() {
            latin += 1;
        }
    }

    if ru_specific > 0 {
        "uk"
    } else if uk_specific > 0 {
        "en"
    } else if cyrillic > latin && cyrillic > 0 {
        "en"
    } else {
        "uk"
    }
}

pub fn get_lang_label(code: &str) -> &'static str {
    match code {
        "auto" => "Автовизначення",
        "uk"   => "Українська",
        "en"   => "Англійська",
        "pl"   => "Польська",
        "ru"   => "Російська",
        "de"   => "Німецька",
        "fr"   => "Французька",
        "es"   => "Іспанська",
        "it"   => "Італійська",
        "pt"   => "Португальська",
        "tr"   => "Турецька",
        "nl"   => "Нідерландська",
        "cs"   => "Чеська",
        "sk"   => "Словацька",
        "ro"   => "Румунська",
        "hu"   => "Угорська",
        "el"   => "Грецька",
        "bg"   => "Болгарська",
        "sv"   => "Шведська",
        "no"   => "Норвезька",
        "da"   => "Данська",
        "fi"   => "Фінська",
        "lt"   => "Литовська",
        "lv"   => "Латвійська",
        "et"   => "Естонська",
        "ja"   => "Японська",
        "zh"   => "Китайська",
        "ko"   => "Корейська",
        "ar"   => "Арабська",
        "he"   => "Іврит",
        "hi"   => "Гінді",
        _      => "Мова",
    }
}

pub fn get_lang_label_en(code: &str) -> &'static str {
    match code {
        "auto" => "Auto-detect",
        "uk"   => "Ukrainian",
        "en"   => "English",
        "pl"   => "Polish",
        "ru"   => "Russian",
        "de"   => "German",
        "fr"   => "French",
        "es"   => "Spanish",
        "it"   => "Italian",
        "pt"   => "Portuguese",
        "tr"   => "Turkish",
        "nl"   => "Dutch",
        "cs"   => "Czech",
        "sk"   => "Slovak",
        "ro"   => "Romanian",
        "hu"   => "Hungarian",
        "el"   => "Greek",
        "bg"   => "Bulgarian",
        "sv"   => "Swedish",
        "no"   => "Norwegian",
        "da"   => "Danish",
        "fi"   => "Finnish",
        "lt"   => "Lithuanian",
        "lv"   => "Latvian",
        "et"   => "Estonian",
        "ja"   => "Japanese",
        "zh"   => "Chinese",
        "ko"   => "Korean",
        "ar"   => "Arabic",
        "he"   => "Hebrew",
        "hi"   => "Hindi",
        _      => "Language",
    }
}

// Returns localized language name based on active UI language
pub fn get_lang_label_for(code: &str, ui_lang: &str) -> &'static str {
    if ui_lang == "en" { get_lang_label_en(code) } else { get_lang_label(code) }
}
