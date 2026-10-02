use base64::prelude::*;
use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::{HLOCAL, LocalFree};
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub popup_mode: bool,
    pub always_on_top: bool,
    pub auto_copy: bool,
    pub close_on_paste: bool,
    pub src_lang: String,
    pub dst_lang: String,
    #[serde(default)]
    pub win_x: Option<i32>,
    #[serde(default)]
    pub win_y: Option<i32>,
    #[serde(default)]
    pub priority_langs: Vec<String>,
    #[serde(default = "default_ui_lang")]
    pub ui_lang: String,
    #[serde(default = "default_true")]
    pub enable_alt_c: bool,
    #[serde(default = "default_true")]
    pub enable_alt_x: bool,
    #[serde(default)]
    pub custom_translate_shortcut: String,
    #[serde(default = "default_false")]
    pub enable_custom_translate: bool,
    #[serde(default)]
    pub custom_window_shortcut: String,
    #[serde(default = "default_false")]
    pub enable_custom_window: bool,
    #[serde(default)]
    pub custom_shortcut: String,
    #[serde(default = "default_false")]
    pub enable_custom_shortcut: bool,
    #[serde(default = "default_custom_action")]
    pub custom_action: String,
    #[serde(default = "default_true")]
    pub enable_ctrl_alt_c: bool,
    #[serde(default = "default_false")]
    pub autostart: bool,
    #[serde(default = "default_true")]
    pub use_deepl: bool,
    #[serde(default = "default_deepl_key")]
    pub deepl_key: String,
}

fn default_ui_lang() -> String { "uk".to_string() }
fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_custom_action() -> String { "translate".to_string() }
fn default_deepl_key() -> String { String::new() }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            popup_mode: true,
            always_on_top: true,
            auto_copy: false,
            close_on_paste: true,
            src_lang: "auto".to_string(),
            dst_lang: "uk".to_string(),
            win_x: None,
            win_y: None,
            priority_langs: vec!["uk".to_string(), "en".to_string()],
            ui_lang: "uk".to_string(),
            enable_alt_c: true,
            enable_alt_x: true,
            custom_translate_shortcut: String::new(),
            enable_custom_translate: false,
            custom_window_shortcut: String::new(),
            enable_custom_window: false,
            custom_shortcut: String::new(),
            enable_custom_shortcut: false,
            custom_action: "translate".to_string(),
            enable_ctrl_alt_c: true,
            autostart: false,
            use_deepl: true,
            deepl_key: String::new(),
        }
    }
}

pub fn dpapi_encrypt(data: &str) -> Result<String, String> {
    if data.trim().is_empty() {
        return Ok(String::new());
    }
    let bytes = data.as_bytes();
    let input_blob = CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let mut output_blob = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptProtectData(
            &input_blob,
            windows::core::PCWSTR::null(),
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output_blob,
        ).map_err(|e| format!("CryptProtectData error: {:?}", e))?;

        let slice = std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize);
        let encoded = BASE64_STANDARD.encode(slice);
        let _ = LocalFree(Some(HLOCAL(output_blob.pbData as _)));

        Ok(format!("dpapi:{}", encoded))
    }
}

pub fn dpapi_decrypt(encrypted: &str) -> Result<String, String> {
    if encrypted.trim().is_empty() {
        return Ok(String::new());
    }
    let b64 = encrypted.strip_prefix("dpapi:").unwrap_or(encrypted);
    let cipher_bytes = BASE64_STANDARD.decode(b64).map_err(|e| format!("Base64 decode error: {:?}", e))?;

    let input_blob = CRYPT_INTEGER_BLOB {
        cbData: cipher_bytes.len() as u32,
        pbData: cipher_bytes.as_ptr() as *mut u8,
    };
    let mut output_blob = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptUnprotectData(
            &input_blob,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output_blob,
        ).map_err(|e| format!("CryptUnprotectData error: {:?}", e))?;

        let slice = std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize);
        let plaintext = String::from_utf8_lossy(slice).to_string();
        let _ = LocalFree(Some(HLOCAL(output_blob.pbData as _)));

        Ok(plaintext)
    }
}

impl AppConfig {
    pub fn config_path() -> std::path::PathBuf {
        // 1. If config.json already exists next to the .exe, always use it (Portable mode)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(dir) = exe_path.parent() {
                let local_path = dir.join("config.json");
                if local_path.exists() {
                    return local_path;
                }

                // 2. If not created yet, check if the folder is writable
                let test_file = dir.join(".write_test");
                if std::fs::write(&test_file, b"").is_ok() {
                    let _ = std::fs::remove_file(&test_file);
                    return local_path;
                }
            }
        }

        // 3. If the .exe folder is write-protected (e.g. Program Files), store in %APPDATA%\LouiseTranslator
        if let Ok(appdata) = std::env::var("APPDATA") {
            let app_dir = std::path::PathBuf::from(appdata).join("LouiseTranslator");
            let _ = std::fs::create_dir_all(&app_dir);
            return app_dir.join("config.json");
        }

        std::path::PathBuf::from("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&data) {
                // Migrate legacy single custom_shortcut (one-time migration)
                if cfg.custom_translate_shortcut.is_empty() && cfg.custom_window_shortcut.is_empty() && !cfg.custom_shortcut.is_empty() {
                    if cfg.custom_action == "toggle_window" {
                        cfg.custom_window_shortcut = cfg.custom_shortcut.clone();
                        cfg.enable_custom_window = cfg.enable_custom_shortcut;
                    } else {
                        cfg.custom_translate_shortcut = cfg.custom_shortcut.clone();
                        cfg.enable_custom_translate = cfg.enable_custom_shortcut;
                    }
                    cfg.custom_shortcut.clear();
                    cfg.enable_custom_shortcut = false;
                    cfg.save();
                }
                if cfg.deepl_key.starts_with("dpapi:") {
                    match dpapi_decrypt(&cfg.deepl_key) {
                        Ok(plain) => cfg.deepl_key = plain,
                        Err(e) => {
                            eprintln!("Failed to decrypt DeepL key: {}", e);
                            cfg.deepl_key.clear();
                        }
                    }
                } else if !cfg.deepl_key.trim().is_empty() {
                    // Automatically migrate plaintext key to DPAPI encrypted storage
                    cfg.save();
                }
                return cfg;
            }
        }
        let default_cfg = Self::default();
        default_cfg.save();
        default_cfg
    }

    pub fn save(&self) {
        let path = Self::config_path();
        let mut to_save = self.clone();
        if !to_save.deepl_key.trim().is_empty() && !to_save.deepl_key.starts_with("dpapi:") {
            if let Ok(enc) = dpapi_encrypt(&to_save.deepl_key) {
                to_save.deepl_key = enc;
            }
        }
        if let Ok(data) = serde_json::to_string_pretty(&to_save) {
            let _ = std::fs::write(&path, data);
        }
    }
}
