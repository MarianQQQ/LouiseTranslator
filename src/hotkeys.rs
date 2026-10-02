use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};
use crate::config::AppConfig;
use crate::AppWindow;

pub static ALT_X_ID: AtomicU32 = AtomicU32::new(0);
pub static CUSTOM_TRANSLATE_ID: AtomicU32 = AtomicU32::new(0);
pub static CUSTOM_WINDOW_ID: AtomicU32 = AtomicU32::new(0);
pub static CUSTOM_HOTKEY_ID: AtomicU32 = AtomicU32::new(0);
pub static CUSTOM_ACTION_IS_TOGGLE: AtomicBool = AtomicBool::new(false);
pub static IS_RECORDING_SHORTCUT: AtomicBool = AtomicBool::new(false);

// Parses a hotkey combination string into a HotKey instance
pub fn parse_custom_hotkey(s: &str) -> Option<HotKey> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Ok(hk) = s.parse::<HotKey>() {
        return Some(hk);
    }
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    if parts.is_empty() {
        return None;
    }
    let mut mods = Modifiers::empty();
    let mut code_str = String::new();
    for part in &parts {
        let p = part.to_lowercase();
        match p.as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" => mods |= Modifiers::ALT,
            "shift" => mods |= Modifiers::SHIFT,
            "win" | "super" | "meta" => mods |= Modifiers::SUPER,
            _ => code_str = p,
        }
    }
    let code = match code_str.as_str() {
        "a" | "ф" => Code::KeyA,
        "b" | "и" => Code::KeyB,
        "c" | "с" => Code::KeyC,
        "d" | "в" => Code::KeyD,
        "e" | "у" => Code::KeyE,
        "f" | "а" => Code::KeyF,
        "g" | "п" | "ґ" => Code::KeyG,
        "h" | "р" => Code::KeyH,
        "i" | "ш" => Code::KeyI,
        "j" | "о" => Code::KeyJ,
        "k" | "л" => Code::KeyK,
        "l" | "д" => Code::KeyL,
        "m" | "ь" => Code::KeyM,
        "n" | "т" => Code::KeyN,
        "o" | "щ" => Code::KeyO,
        "p" | "з" => Code::KeyP,
        "q" | "й" => Code::KeyQ,
        "r" | "к" => Code::KeyR,
        "s" | "і" | "ы" => Code::KeyS,
        "t" | "е" => Code::KeyT,
        "u" | "г" => Code::KeyU,
        "v" | "м" => Code::KeyV,
        "w" | "ц" => Code::KeyW,
        "x" | "ч" => Code::KeyX,
        "y" | "н" => Code::KeyY,
        "z" | "я" => Code::KeyZ,
        "0" => Code::Digit0, "1" => Code::Digit1, "2" => Code::Digit2, "3" => Code::Digit3,
        "4" => Code::Digit4, "5" => Code::Digit5, "6" => Code::Digit6, "7" => Code::Digit7,
        "8" => Code::Digit8, "9" => Code::Digit9,
        "space" => Code::Space,
        "f1" => Code::F1, "f2" => Code::F2, "f3" => Code::F3, "f4" => Code::F4,
        "f5" => Code::F5, "f6" => Code::F6, "f7" => Code::F7, "f8" => Code::F8,
        "f9" => Code::F9, "f10" => Code::F10, "f11" => Code::F11, "f12" => Code::F12,
        _ => return None,
    };
    let m = if mods.is_empty() { None } else { Some(mods) };
    Some(HotKey::new(m, code))
}

// Synchronizes active hotkeys with GlobalHotKeyManager
pub fn sync_registered_hotkeys(
    manager: &GlobalHotKeyManager,
    registered: &mut Vec<HotKey>,
    cfg: &AppConfig,
    app_weak: slint::Weak<AppWindow>,
) {
    ALT_X_ID.store(0, Ordering::Relaxed);
    CUSTOM_TRANSLATE_ID.store(0, Ordering::Relaxed);
    CUSTOM_WINDOW_ID.store(0, Ordering::Relaxed);
    CUSTOM_HOTKEY_ID.store(0, Ordering::Relaxed);
    CUSTOM_ACTION_IS_TOGGLE.store(false, Ordering::Relaxed);
    for hk in registered.drain(..) {
        let _ = manager.unregister(hk);
    }

    let mut failed = Vec::new();

    // 1. Selection translation:
    // If a custom combination is defined, register it instead of Alt+C
    if !cfg.custom_translate_shortcut.trim().is_empty() {
        if let Some(hk) = parse_custom_hotkey(&cfg.custom_translate_shortcut) {
            let id = hk.id();
            if manager.register(hk).is_ok() {
                registered.push(hk);
                CUSTOM_TRANSLATE_ID.store(id, Ordering::Relaxed);
            } else {
                failed.push(cfg.custom_translate_shortcut.clone());
            }
        }
    } else if cfg.enable_alt_c {
        // Otherwise register default Alt+C (instant selection translation)
        let hk = HotKey::new(Some(Modifiers::ALT), Code::KeyC);
        if manager.register(hk).is_ok() {
            registered.push(hk);
        } else {
            failed.push("Alt+C".to_string());
        }
    }

    // 2. Open / toggle window:
    // If a custom combination is defined, register it instead of Alt+X
    if !cfg.custom_window_shortcut.trim().is_empty() {
        if let Some(hk) = parse_custom_hotkey(&cfg.custom_window_shortcut) {
            let id = hk.id();
            if manager.register(hk).is_ok() {
                registered.push(hk);
                CUSTOM_WINDOW_ID.store(id, Ordering::Relaxed);
            } else {
                failed.push(cfg.custom_window_shortcut.clone());
            }
        }
    } else if cfg.enable_alt_x {
        // Otherwise register default Alt+X (toggle window)
        let hk = HotKey::new(Some(Modifiers::ALT), Code::KeyX);
        let id = hk.id();
        if manager.register(hk).is_ok() {
            registered.push(hk);
            ALT_X_ID.store(id, Ordering::Relaxed);
        } else {
            failed.push("Alt+X".to_string());
        }
    }

    // Backward compatibility for legacy custom_shortcut (if new fields are empty)
    if !cfg.enable_custom_translate && !cfg.enable_custom_window && cfg.enable_custom_shortcut && !cfg.custom_shortcut.trim().is_empty() {
        if let Some(hk) = parse_custom_hotkey(&cfg.custom_shortcut) {
            let id = hk.id();
            if manager.register(hk).is_ok() {
                registered.push(hk);
                if cfg.custom_action == "toggle_window" {
                    CUSTOM_WINDOW_ID.store(id, Ordering::Relaxed);
                } else {
                    CUSTOM_TRANSLATE_ID.store(id, Ordering::Relaxed);
                }
            } else {
                failed.push(cfg.custom_shortcut.clone());
            }
        }
    }

    if !failed.is_empty() {
        let msg = if cfg.ui_lang == "en" {
            format!("⚠️ Shortcut {} is occupied by another Windows app. Change it in Settings (⚙).", failed.join(", "))
        } else {
            format!("⚠️ Гарячу клавішу {} зайнято іншою програмою (наприклад, драйвером або месенджером). Змініть її у Налаштуваннях (⚙).", failed.join(", "))
        };
        crate::show_toast(app_weak, msg, "warning", 7000);
    }
}
