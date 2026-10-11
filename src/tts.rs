use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use windows::core::HSTRING;
use windows::Foundation::Uri;
use windows::Media::Core::MediaSource;
use windows::Media::Playback::{MediaPlayer, MediaPlaybackState};

static PLAYBACK_COUNTER: AtomicU64 = AtomicU64::new(0);
static IS_PLAYING: AtomicBool = AtomicBool::new(false);
static ACTIVE_PLAYER: Mutex<Option<MediaPlayer>> = Mutex::new(None);

type TtsCache = Mutex<HashMap<(String, String, bool), Vec<u8>>>;
static TTS_CACHE: std::sync::LazyLock<TtsCache> = std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Maps Louise language code to Google TTS language code.
pub fn map_lang_for_tts(lang: &str) -> &'static str {
    match lang.trim().to_lowercase().as_str() {
        "uk" => "uk",
        "en" => "en",
        "pl" => "pl",
        "de" => "de",
        "fr" => "fr",
        "es" => "es",
        "it" => "it",
        "pt" => "pt",
        "nl" => "nl",
        "cs" => "cs",
        "sk" => "sk",
        "ro" => "ro",
        "hu" => "hu",
        "tr" => "tr",
        "el" => "el",
        "bg" => "bg",
        "sv" => "sv",
        "no" => "no",
        "da" => "da",
        "fi" => "fi",
        "lt" => "lt",
        "lv" => "lv",
        "et" => "et",
        "ja" => "ja",
        "zh" => "zh-CN",
        "ko" => "ko",
        "ar" => "ar",
        "he" => "iw", // Google TTS uses "iw" for Hebrew
        "hi" => "hi",
        "ru" => "ru",
        _ => "en",
    }
}

/// Normalizes text before TTS synthesis to eliminate long unnatural pauses between sentences,
/// paragraphs, dashes, and ellipses while preserving natural conversational flow.
pub fn normalize_text_for_tts(text: &str) -> String {
    let non_empty_lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    let total = non_empty_lines.len();
    let mut merged_lines = Vec::with_capacity(total);
    for (idx, trimmed) in non_empty_lines.into_iter().enumerate() {
        let mut line_str = trimmed.to_string();
        if idx + 1 < total {
            if let Some(last_ch) = line_str.chars().last() {
                if !matches!(last_ch, '.' | '!' | '?' | ',' | ';' | ':') {
                    line_str.push(',');
                }
            }
        }
        merged_lines.push(line_str);
    }

    let mut s = merged_lines.join(" ");
    // Replace ellipses and long dashes (which cause 800-1000ms pauses) with short comma pauses
    s = s.replace("...", ",")
        .replace('…', ",")
        .replace(" — ", ", ")
        .replace(" – ", ", ")
        .replace(" - ", ", ")
        .replace("; ", ", ")
        .replace(": ", ", ");

    // Replace mid-text sentence periods (". ") with ", " so TTS takes a brief ~180ms breath
    // instead of a 750ms full stop between sentences, while keeping final punctuation intact
    while s.contains(". ") {
        s = s.replace(". ", ", ");
    }
    while s.contains(",,") {
        s = s.replace(",,", ",");
    }

    // Collapse multiple whitespace characters
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Splits text into chunks of at most `max_len` UTF-8 characters without breaking words or flushing on newlines.
pub fn split_text_for_tts(text: &str, max_len: usize) -> Vec<String> {
    let clean = normalize_text_for_tts(text);
    if clean.is_empty() {
        return Vec::new();
    }
    if clean.chars().count() <= max_len {
        return vec![clean];
    }

    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for word in clean.split_whitespace() {
        if current_chunk.is_empty() {
            current_chunk.push_str(word);
        } else if current_chunk.chars().count() + 1 + word.chars().count() <= max_len {
            current_chunk.push(' ');
            current_chunk.push_str(word);
        } else {
            chunks.push(current_chunk);
            current_chunk = word.to_string();
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

/// Percent-encodes a string for URL query parameters without external dependencies.
pub fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 2);
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

/// Fetches MP3 audio from Google Translate TTS with caching and multi-chunk support.
use kothok_edge_tts::{init_tls, EdgeTts, Engine, TtsEvent};

static TLS_INITIALIZED: std::sync::Once = std::sync::Once::new();

/// Structure representing Edge TTS voice profile.
pub struct EdgeVoice {
    pub voice: &'static str,
    pub locale: &'static str,
}

/// Maps language code to the best Microsoft Edge neural voice.
/// Defaults to male voice Ostap for Ukrainian, and high-quality neural voices for others.
pub fn map_lang_for_edge_tts(lang: &str) -> EdgeVoice {
    match lang.trim().to_lowercase().as_str() {
        "uk" => EdgeVoice {
            voice: "uk-UA-OstapNeural",
            locale: "uk-UA",
        },
        "en" => EdgeVoice {
            voice: "en-US-ChristopherNeural",
            locale: "en-US",
        },
        "pl" => EdgeVoice {
            voice: "pl-PL-MarekNeural",
            locale: "pl-PL",
        },
        "de" => EdgeVoice {
            voice: "de-DE-KillianNeural",
            locale: "de-DE",
        },
        "fr" => EdgeVoice {
            voice: "fr-FR-HenriNeural",
            locale: "fr-FR",
        },
        "es" => EdgeVoice {
            voice: "es-ES-AlvaroNeural",
            locale: "es-ES",
        },
        "it" => EdgeVoice {
            voice: "it-IT-DiegoNeural",
            locale: "it-IT",
        },
        "pt" => EdgeVoice {
            voice: "pt-BR-AntonioNeural",
            locale: "pt-BR",
        },
        "nl" => EdgeVoice {
            voice: "nl-NL-MaartenNeural",
            locale: "nl-NL",
        },
        "cs" => EdgeVoice {
            voice: "cs-CZ-AntoninNeural",
            locale: "cs-CZ",
        },
        "sk" => EdgeVoice {
            voice: "sk-SK-LukasNeural",
            locale: "sk-SK",
        },
        "ro" => EdgeVoice {
            voice: "ro-RO-EmilNeural",
            locale: "ro-RO",
        },
        "hu" => EdgeVoice {
            voice: "hu-HU-TamasNeural",
            locale: "hu-HU",
        },
        "tr" => EdgeVoice {
            voice: "tr-TR-AhmetNeural",
            locale: "tr-TR",
        },
        "el" => EdgeVoice {
            voice: "el-GR-NestorasNeural",
            locale: "el-GR",
        },
        "bg" => EdgeVoice {
            voice: "bg-BG-BorislavNeural",
            locale: "bg-BG",
        },
        "sv" => EdgeVoice {
            voice: "sv-SE-MattiasNeural",
            locale: "sv-SE",
        },
        "no" => EdgeVoice {
            voice: "nb-NO-FinnNeural",
            locale: "nb-NO",
        },
        "da" => EdgeVoice {
            voice: "da-DK-JeppeNeural",
            locale: "da-DK",
        },
        "fi" => EdgeVoice {
            voice: "fi-FI-HarriNeural",
            locale: "fi-FI",
        },
        "lt" => EdgeVoice {
            voice: "lt-LT-LeonasNeural",
            locale: "lt-LT",
        },
        "lv" => EdgeVoice {
            voice: "lv-LV-NilsNeural",
            locale: "lv-LV",
        },
        "et" => EdgeVoice {
            voice: "et-EE-KertNeural",
            locale: "et-EE",
        },
        "ja" => EdgeVoice {
            voice: "ja-JP-KeitaNeural",
            locale: "ja-JP",
        },
        "zh" => EdgeVoice {
            voice: "zh-CN-YunxiNeural",
            locale: "zh-CN",
        },
        "ko" => EdgeVoice {
            voice: "ko-KR-InJoonNeural",
            locale: "ko-KR",
        },
        "ar" => EdgeVoice {
            voice: "ar-EG-ShakirNeural",
            locale: "ar-EG",
        },
        "he" => EdgeVoice {
            voice: "he-IL-AvriNeural",
            locale: "he-IL",
        },
        "hi" => EdgeVoice {
            voice: "hi-IN-MadhurNeural",
            locale: "hi-IN",
        },
        "ru" => EdgeVoice {
            voice: "ru-RU-DmitryNeural",
            locale: "ru-RU",
        },
        _ => EdgeVoice {
            voice: "en-US-ChristopherNeural",
            locale: "en-US",
        },
    }
}

/// Synthesizes speech using Microsoft Edge Neural TTS with natural conversational rate (+3%) and reduced sentence pauses.
pub async fn fetch_edge_tts_audio(text: &str, lang: &str) -> Result<Vec<u8>, String> {
    TLS_INITIALIZED.call_once(|| {
        init_tls();
    });

    let normalized = normalize_text_for_tts(text);
    let synth_text = if normalized.is_empty() { text } else { &normalized };

    let voice_spec = map_lang_for_edge_tts(lang);
    let events = EdgeTts
        .synthesize(synth_text, voice_spec.voice, "+3%", voice_spec.locale)
        .await
        .map_err(|e| format!("Edge TTS synthesis error: {}", e))?;

    let mut audio_data = Vec::new();
    for ev in events {
        if let TtsEvent::Audio(bytes) = ev {
            audio_data.extend_from_slice(&bytes);
        }
    }

    if audio_data.is_empty() {
        return Err("Edge TTS returned empty audio".to_string());
    }

    Ok(audio_data)
}

/// Fallback Google TTS fetcher.
pub async fn fetch_google_tts_audio(clean_text: &str, tts_lang: &str) -> Result<Vec<u8>, String> {
    let chunks = split_text_for_tts(clean_text, 180);
    if chunks.is_empty() {
        return Err("No text to synthesize".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|e| format!("HTTP Client error: {}", e))?;

    let mut combined_audio = Vec::new();

    for chunk in chunks {
        let url = format!(
            "https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl={}&q={}",
            tts_lang,
            url_encode(&chunk)
        );

        let res = client
            .get(&url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            )
            .send()
            .await
            .map_err(|e| format!("TTS network request failed: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("TTS server returned HTTP {}", res.status()));
        }

        let bytes = res.bytes().await.map_err(|e| format!("TTS audio decode error: {}", e))?;
        combined_audio.extend_from_slice(&bytes);
    }

    Ok(combined_audio)
}

/// Fetches MP3 audio from Edge Neural TTS (with automatic fallback to Google TTS) or Google TTS directly, with RAM caching.
pub async fn fetch_tts_audio(text: &str, lang: &str, use_edge_tts: bool) -> Result<Vec<u8>, String> {
    let clean_text = text.trim();
    if clean_text.is_empty() {
        return Err("Text is empty".to_string());
    }

    let cache_key = (clean_text.to_string(), lang.trim().to_lowercase(), use_edge_tts);

    // Check cache
    {
        let cache = TTS_CACHE.lock().unwrap();
        if let Some(cached_bytes) = cache.get(&cache_key) {
            return Ok(cached_bytes.clone());
        }
    }

    let audio_res = if use_edge_tts {
        // Primary: Microsoft Edge Neural TTS with fallback to Google TTS
        match tokio::time::timeout(Duration::from_secs(8), fetch_edge_tts_audio(clean_text, lang)).await {
            Ok(Ok(bytes)) => Ok(bytes),
            Ok(Err(err)) => {
                eprintln!("Edge TTS failed ({}), falling back to Google TTS...", err);
                let google_lang = map_lang_for_tts(lang);
                fetch_google_tts_audio(clean_text, google_lang).await
            }
            Err(_) => {
                eprintln!("Edge TTS timed out, falling back to Google TTS...",);
                let google_lang = map_lang_for_tts(lang);
                fetch_google_tts_audio(clean_text, google_lang).await
            }
        }
    } else {
        // Primary: Google TTS with fallback to Microsoft Edge Neural TTS
        let google_lang = map_lang_for_tts(lang);
        match fetch_google_tts_audio(clean_text, google_lang).await {
            Ok(bytes) => Ok(bytes),
            Err(err) => {
                eprintln!("Google TTS failed ({}), falling back to Edge TTS...", err);
                match tokio::time::timeout(Duration::from_secs(8), fetch_edge_tts_audio(clean_text, lang)).await {
                    Ok(res) => res,
                    Err(_) => Err(format!("Google TTS failed ({}) and Edge TTS timed out", err)),
                }
            }
        }
    };

    let audio_bytes = audio_res?;

    // Save to cache (limit size to 30 items)
    {
        let mut cache = TTS_CACHE.lock().unwrap();
        if cache.len() >= 30 {
            cache.clear();
        }
        cache.insert(cache_key, audio_bytes.clone());
    }

    Ok(audio_bytes)
}

/// Returns true if audio is currently playing.
pub fn is_tts_playing() -> bool {
    IS_PLAYING.load(Ordering::Relaxed)
}

/// Stops any currently active TTS playback.
pub fn stop_tts() {
    IS_PLAYING.store(false, Ordering::SeqCst);
    if let Ok(mut guard) = ACTIVE_PLAYER.lock() {
        if let Some(player) = guard.take() {
            let _ = player.Pause();
            // Drop player to free all audio session resources
        }
    }
}

/// Plays audio bytes through native Windows MediaPlayer subsystem, invoking `on_state_changed` when starting and finishing.
pub fn play_tts_bytes(
    audio_bytes: Vec<u8>,
    on_state_changed: Arc<dyn Fn(bool) + Send + Sync>,
) -> Result<(), String> {
    // 1. Stop any ongoing playback
    stop_tts();

    let current_id = PLAYBACK_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;

    // 2. Write bytes to temporary MP3 file
    let temp_path = std::env::temp_dir().join(format!("louise_tts_{}.mp3", current_id));
    fs::write(&temp_path, &audio_bytes).map_err(|e| format!("Failed to write temp audio file: {}", e))?;

    let abs_path = temp_path.to_string_lossy().to_string();
    let uri_string = format!("file:///{}", abs_path.replace('\\', "/"));

    // 3. Create Windows MediaPlayer and MediaSource from file URI
    let uri_h = HSTRING::from(&uri_string);
    let uri = Uri::CreateUri(&uri_h).map_err(|e| format!("URI error: {}", e))?;
    let source = MediaSource::CreateFromUri(&uri).map_err(|e| format!("MediaSource error: {}", e))?;

    let player = MediaPlayer::new().map_err(|e| format!("MediaPlayer error: {}", e))?;
    player.SetSource(&source).map_err(|e| format!("SetSource error: {}", e))?;

    // Store active player in global state
    {
        let mut guard = ACTIVE_PLAYER.lock().unwrap();
        *guard = Some(player.clone());
    }

    IS_PLAYING.store(true, Ordering::SeqCst);
    on_state_changed(true);

    player.Play().map_err(|e| format!("Play error: {}", e))?;

    // 4. Monitor playback completion in background thread
    let callback = on_state_changed.clone();
    let player_clone = player.clone();
    let temp_path_clone = temp_path.clone();
    std::thread::spawn(move || {
        let mut has_started = false;
        loop {
            std::thread::sleep(Duration::from_millis(100));

            // Check if cancelled or new audio started
            if PLAYBACK_COUNTER.load(Ordering::SeqCst) != current_id || !IS_PLAYING.load(Ordering::Relaxed) {
                let _ = fs::remove_file(&temp_path_clone);
                break;
            }

            if let Ok(session) = player_clone.PlaybackSession() {
                if let Ok(state) = session.PlaybackState() {
                    if state == MediaPlaybackState::Playing || state == MediaPlaybackState::Buffering {
                        has_started = true;
                    } else if has_started && (state == MediaPlaybackState::Paused || state == MediaPlaybackState::None) {
                        // Playback naturally ended
                        IS_PLAYING.store(false, Ordering::SeqCst);
                        if let Ok(mut guard) = ACTIVE_PLAYER.lock() {
                            *guard = None;
                        }
                        let _ = fs::remove_file(&temp_path_clone);
                        callback(false);
                        break;
                    }
                }
            } else {
                break;
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_lang_for_tts() {
        assert_eq!(map_lang_for_tts("uk"), "uk");
        assert_eq!(map_lang_for_tts("en"), "en");
        assert_eq!(map_lang_for_tts("pl"), "pl");
        assert_eq!(map_lang_for_tts("de"), "de");
        assert_eq!(map_lang_for_tts("zh"), "zh-CN");
        assert_eq!(map_lang_for_tts("he"), "iw");
        assert_eq!(map_lang_for_tts("unknown"), "en");
    }

    #[test]
    fn test_split_text_for_tts() {
        assert_eq!(split_text_for_tts("", 100), Vec::<String>::new());
        assert_eq!(split_text_for_tts("Hello world", 100), vec!["Hello world".to_string()]);

        let long = "One two three four five six seven eight nine ten eleven twelve";
        let chunks = split_text_for_tts(long, 20);
        assert!(chunks.len() >= 3);
        for chunk in &chunks {
            assert!(chunk.chars().count() <= 20);
        }
    }

    #[test]
    fn test_media_player_creation() {
        let player = MediaPlayer::new();
        assert!(player.is_ok());
    }

    #[test]
    fn test_map_lang_for_edge_tts() {
        let ua = map_lang_for_edge_tts("uk");
        assert_eq!(ua.voice, "uk-UA-OstapNeural");
        assert_eq!(ua.locale, "uk-UA");

        let en = map_lang_for_edge_tts("en");
        assert_eq!(en.voice, "en-US-ChristopherNeural");

        let pl = map_lang_for_edge_tts("pl");
        assert_eq!(pl.voice, "pl-PL-MarekNeural");
    }

    #[test]
    fn test_normalize_text_for_tts() {
        let raw = "Перше речення.\nДруге речення — з тире...\nТретій рядок без крапки\nЧетвертий рядок.";
        let norm = normalize_text_for_tts(raw);
        assert_eq!(
            norm,
            "Перше речення, Друге речення, з тире, Третій рядок без крапки, Четвертий рядок."
        );
    }
}
