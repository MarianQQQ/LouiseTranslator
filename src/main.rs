#![windows_subsystem = "windows"]

slint::include_modules!();

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use arboard::Clipboard;
use global_hotkey::{
    hotkey::HotKey,
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use slint::ComponentHandle;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, MouseButton, TrayIconBuilder, TrayIconEvent,
};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    ERROR_ALREADY_EXISTS, GetLastError, SetLastError, HWND, POINT,
    WIN32_ERROR,
};
use slint::winit_030::winit::platform::windows::WindowAttributesExtWindows;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::System::Threading::{CreateMutexW, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    VK_CONTROL, VK_LBUTTON, VK_V,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetCursorPos, GetForegroundWindow, SetForegroundWindow,
    ShowWindow, SW_RESTORE,
};

mod config;
pub use config::AppConfig;
mod translate;
pub use translate::*;
mod hotkeys;
pub use hotkeys::*;
mod win32;
pub use win32::*;
mod ocr;
pub use ocr::*;




// Sets status text and clears it automatically after 2.5 seconds
fn set_status_timed(app_weak: slint::Weak<AppWindow>, msg: String) {
    if let Some(app) = app_weak.upgrade() {
        app.set_status_text(msg.clone().into());
    }
    let app_w = app_weak.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(2500));
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = app_w.upgrade() {
                // Clear only if the message has not changed
                if app.get_status_text().to_string() == msg {
                    app.set_status_text("".into());
                }
            }
        });
    });
}

// Displays floating notification toast with automatic dismissal
pub fn show_toast(app_weak: slint::Weak<AppWindow>, msg: String, toast_type: &str, duration_ms: u64) {
    let t_type = toast_type.to_string();
    let msg_clone = msg.clone();
    let app_w = app_weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(app) = app_w.upgrade() {
            app.set_toast_text(msg.into());
            app.set_toast_type(t_type.into());
            app.set_show_toast(true);
        }
    });

    if duration_ms > 0 {
        let app_w_timer = app_weak.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(duration_ms));
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = app_w_timer.upgrade() {
                    if app.get_toast_text().to_string() == msg_clone {
                        app.set_show_toast(false);
                    }
                }
            });
        });
    }
}



// Sets priority languages into slots prio1..prio6 on AppWindow
fn apply_priority_langs(app: &AppWindow, langs: &[String]) {
    let get = |i: usize| langs.get(i).map(|s| s.as_str()).unwrap_or("").into();
    app.set_prio1(get(0));
    app.set_prio2(get(1));
    app.set_prio3(get(2));
    app.set_prio4(get(3));
    app.set_prio5(get(4));
    app.set_prio6(get(5));
}


#[tokio::main]
async fn main() {
    std::panic::set_hook(Box::new(|info| {
        let panic_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("panic.log")))
            .unwrap_or_else(|| std::path::PathBuf::from("panic.log"));
        let msg = format!("PANIC occurred: {}\nLocation: {:?}\nPayload: {:?}", info, info.location(), info.payload());
        let _ = std::fs::write(panic_path, msg);
    }));

    let log_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("startup.log")))
        .unwrap_or_else(|| std::path::PathBuf::from("startup.log"));
    let log = |s: &str| {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
            let _ = writeln!(f, "{}", s);
        }
    };
    let _ = std::fs::remove_file(&log_path);
    log("=== Louise Translator Starting ===");
    if let Err(e) = run_app(&log).await {
        log(&format!("FATAL ERROR: {:?}", e));
    } else {
        log("Application exited normally.");
    }
}

async fn run_app(log: &impl Fn(&str)) -> Result<(), Box<dyn std::error::Error>> {
    unsafe { let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED); }
    log("Checking single instance mutex...");
    // 0. Single instance protection
    unsafe { SetLastError(WIN32_ERROR(0)) };
    let mutex_name: Vec<u16> = "Local\\LouiseTranslator_SingleInstance_Mutex\0".encode_utf16().collect();
    let _single_mutex = match unsafe { CreateMutexW(None, true, PCWSTR(mutex_name.as_ptr())) } {
        Ok(handle) => {
            if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
                log("Another instance is already running (ERROR_ALREADY_EXISTS). Exiting.");
                // If another instance is running, find its window, restore and focus it
                unsafe {
                    let win_name: Vec<u16> = "Louise Translator\0".encode_utf16().collect();
                    if let Ok(existing_hwnd) = FindWindowW(None, PCWSTR(win_name.as_ptr())) {
                        if !existing_hwnd.0.is_null() {
                            let _ = ShowWindow(existing_hwnd, SW_RESTORE);
                            let _ = SetForegroundWindow(existing_hwnd);
                        }
                    }
                }
                std::process::exit(0);
            }
            Some(handle)
        }
        Err(_) => None,
    };

    log("Single instance check passed.");
    log("Loading initial config...");
    let initial_config = AppConfig::load();
    log("Config loaded.");
    let popup_mode = initial_config.popup_mode;
    POPUP_MODE.store(popup_mode, std::sync::atomic::Ordering::Relaxed);
    ALWAYS_ON_TOP.store(initial_config.always_on_top, std::sync::atomic::Ordering::Relaxed);

    log("Checking hidden owner window...");
    let owner = if popup_mode {
        let hwnd = get_or_create_hidden_owner();
        log(&format!("Hidden owner HWND: {:?}", hwnd.0));
        if !hwnd.0.is_null() {
            Some(hwnd.0 as isize)
        } else {
            None
        }
    } else {
        None
    };

    log("Configuring Slint backend with Winit window attributes hook...");
    let sel_res = slint::BackendSelector::new()
        .backend_name("winit".into())
        .with_winit_window_attributes_hook(move |attrs| {
            match owner {
                Some(owner_hwnd) => attrs.with_owner_window(owner_hwnd).with_skip_taskbar(true),
                None => attrs,
            }
        })
        .select();
    log(&format!("BackendSelector select result: {:?}", sel_res));

    log("Creating AppWindow...");
    MAIN_THREAD_ID.store(unsafe { GetCurrentThreadId() }, std::sync::atomic::Ordering::Relaxed);
    let app = match AppWindow::new() {
        Ok(a) => {
            log("AppWindow created successfully.");
            a
        }
        Err(e) => {
            log(&format!("AppWindow::new FAILED: {:?}", e));
            return Err(Box::new(e));
        }
    };
    cache_hwnd_from_window(app.window());

    let prev_window: Arc<Mutex<Option<isize>>> = Arc::new(Mutex::new(None));
    let config: Arc<Mutex<AppConfig>> = Arc::new(Mutex::new(initial_config));
    let drag_state: Arc<Mutex<Option<(i32, i32, i32, i32)>>> = Arc::new(Mutex::new(None));
    // Debounce counter for text input translation (400ms)
    let debounce_seq: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));


    // Load saved user configuration
    {
        let cfg = config.lock().unwrap();
        apply_window_styles(cfg.popup_mode, cfg.always_on_top);
        app.set_popup_mode(cfg.popup_mode);
        app.set_pin_on_top(cfg.always_on_top);
        app.set_auto_copy(cfg.auto_copy);
        app.set_close_on_paste(cfg.close_on_paste);
        app.set_src_lang(cfg.src_lang.clone().into());
        app.set_dst_lang(cfg.dst_lang.clone().into());
        app.set_src_lang_label(get_lang_label_for(&cfg.src_lang, &cfg.ui_lang).into());
        app.set_dst_lang_label(get_lang_label_for(&cfg.dst_lang, &cfg.ui_lang).into());
        app.set_ui_lang(cfg.ui_lang.clone().into());
        let has_custom_tr = !cfg.custom_translate_shortcut.trim().is_empty();
        let has_custom_win = !cfg.custom_window_shortcut.trim().is_empty();
        app.set_enable_alt_c(cfg.enable_alt_c && !has_custom_tr);
        app.set_enable_alt_x(cfg.enable_alt_x && !has_custom_win);
        app.set_custom_translate_shortcut(cfg.custom_translate_shortcut.clone().into());
        app.set_enable_custom_translate(has_custom_tr);
        app.set_custom_window_shortcut(cfg.custom_window_shortcut.clone().into());
        app.set_enable_custom_window(has_custom_win);
        app.set_active_custom_tab(cfg.custom_action.clone().into());
        app.set_custom_shortcut(cfg.custom_shortcut.clone().into());
        app.set_enable_custom_shortcut(cfg.enable_custom_shortcut);
        app.set_custom_action(cfg.custom_action.clone().into());
        app.set_autostart(is_autostart_enabled() || cfg.autostart);
        app.set_use_deepl(cfg.use_deepl);
        app.set_deepl_key(cfg.deepl_key.clone().into());
        let initial_engine = if cfg.use_deepl && !cfg.deepl_key.trim().is_empty() {
            "DeepL ⚡"
        } else {
            "Google"
        };
        app.set_engine_label(initial_engine.into());
        apply_priority_langs(&app, &cfg.priority_langs);
        if cfg.use_deepl && !cfg.deepl_key.trim().is_empty() {
            update_deepl_quota_display(app.as_weak(), cfg.deepl_key.clone());
        }
    }

    // 0. Initialize Windows System Tray
    log("Resolving tray icon...");
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let ico_path = exe_dir.map(|d| d.join("app_icon.ico"));
    let tray_icon = Icon::from_resource(1, Some((32, 32)))
        .or_else(|_| {
            ico_path
                .as_ref()
                .map(|p| Icon::from_path(p, Some((32, 32))))
                .unwrap_or_else(|| Icon::from_path("app_icon.ico", Some((32, 32))))
        })
        .or_else(|_| Icon::from_path("app_icon.ico", Some((32, 32))))
        .or_else(|_| {
            let icon_rgba = include_bytes!("../ui/icon_32.rgba");
            Icon::from_rgba(icon_rgba.to_vec(), 32, 32)
        })
        .expect("valid icon");
    log("Tray icon resolved.");

    let is_en = {
        let cfg = config.lock().unwrap();
        cfg.ui_lang == "en"
    };
    let tray_menu = Menu::new();
    let show_item = MenuItem::new(if is_en { "Open Louise Translator" } else { "Відкрити Louise Translator" }, true, None);
    let settings_item = MenuItem::new(if is_en { "Settings ⚙" } else { "Налаштування ⚙" }, true, None);
    let quit_item = MenuItem::new(if is_en { "Exit" } else { "Вихід" }, true, None);
    let _ = tray_menu.append_items(&[&show_item, &settings_item, &quit_item]);

    let show_id = show_item.id().clone();
    let settings_id = settings_item.id().clone();
    let quit_id = quit_item.id().clone();

    log("Building TrayIcon...");
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_menu_on_left_click(false)
        .with_tooltip("Louise Translator")
        .with_icon(tray_icon)
        .build()?;
    let _ = tray.set_visible(true);
    let _tray_static = Box::leak(Box::new(tray));
    log("TrayIcon built and set visible.");

    let app_weak_tray = app.as_weak();
    std::thread::spawn(move || {
        let tray_channel = TrayIconEvent::receiver();
        let menu_channel = MenuEvent::receiver();
        let mut last_click_time = std::time::Instant::now() - Duration::from_secs(10);
        loop {
            let mut got_event = false;

            while let Ok(event) = tray_channel.try_recv() {
                got_event = true;
                match event {
                    TrayIconEvent::Click { button: MouseButton::Left, .. } => {
                        let now = std::time::Instant::now();
                        if now.duration_since(last_click_time) < Duration::from_millis(200) {
                            continue;
                        }
                        last_click_time = now;

                        let app_w = app_weak_tray.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(app) = app_w.upgrade() {
                                show_app_window(&app);
                                position_window_at_cursor(&app);
                                app.set_show_settings(false);
                                app.invoke_focus_input();
                            }
                        });
                    }
                    _ => {}
                }
            }

            while let Ok(event) = menu_channel.try_recv() {
                got_event = true;
                if event.id == show_id {
                    let app_w = app_weak_tray.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(app) = app_w.upgrade() {
                            show_app_window(&app);
                            position_window_at_cursor(&app);
                            app.set_show_settings(false);
                            app.invoke_focus_input();
                        }
                    });
                } else if event.id == settings_id {
                    let app_w = app_weak_tray.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(app) = app_w.upgrade() {
                            show_app_window(&app);
                            app.set_show_settings(true);
                            let key = app.get_deepl_key().to_string();
                            update_deepl_quota_display(app_w.clone(), key);
                        }
                    });
                } else if event.id == quit_id {
                    let _ = slint::quit_event_loop();
                }
            }

            if !got_event {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    });

    // 1. Global hotkey registration with dynamic synchronization
    log("Registering hotkeys...");
    let hotkey_manager = Arc::new(Mutex::new(GlobalHotKeyManager::new()?));
    let registered_keys: Arc<Mutex<Vec<HotKey>>> = Arc::new(Mutex::new(Vec::new()));

    {
        let mgr = hotkey_manager.lock().unwrap();
        let mut reg = registered_keys.lock().unwrap();
        let cfg = config.lock().unwrap();
        sync_registered_hotkeys(&mgr, &mut reg, &cfg, app.as_weak());
    }
    log("Hotkeys registered successfully.");

    let app_weak = app.as_weak();
    let prev_window_clone = prev_window.clone();

    // 2. Background hotkey event listener
    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.recv() {
            if IS_RECORDING_SHORTCUT.load(Ordering::Relaxed) {
                continue;
            }
            if event.state == global_hotkey::HotKeyState::Pressed {
                let current_fg = unsafe { GetForegroundWindow() };
                *prev_window_clone.lock().unwrap() = Some(current_fg.0 as isize);

                let alt_x_id = ALT_X_ID.load(std::sync::atomic::Ordering::Relaxed);
                let custom_win_id = CUSTOM_WINDOW_ID.load(std::sync::atomic::Ordering::Relaxed);
                let is_open_only = (alt_x_id != 0 && event.id == alt_x_id)
                    || (custom_win_id != 0 && event.id == custom_win_id);

                if is_open_only {
                    // Release Alt key state in the system
                    unsafe {
                        let cancel_menu = [
                            INPUT {
                                r#type: INPUT_KEYBOARD,
                                Anonymous: INPUT_0 {
                                    ki: KEYBDINPUT {
                                        wVk: VK_CONTROL,
                                        wScan: 0,
                                        dwFlags: Default::default(),
                                        time: 0,
                                        dwExtraInfo: 0,
                                    },
                                },
                            },
                            INPUT {
                                r#type: INPUT_KEYBOARD,
                                Anonymous: INPUT_0 {
                                    ki: KEYBDINPUT {
                                        wVk: VK_CONTROL,
                                        wScan: 0,
                                        dwFlags: KEYEVENTF_KEYUP,
                                        time: 0,
                                        dwExtraInfo: 0,
                                    },
                                },
                            },
                        ];
                        let _ = SendInput(&cancel_menu, std::mem::size_of::<INPUT>() as i32);
                    }

                    let is_shown = is_window_shown();
                    let app_weak_inner = app_weak.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(app) = app_weak_inner.upgrade() {
                            if is_shown {
                                // If the window is already open, pressing Alt+X again HIDES it to tray!
                                hide_app_window(&app);
                            } else {
                                // If the window is hidden, OPEN it near the cursor preserving existing text
                                app.set_show_settings(false);
                                if app.get_src_text().trim().is_empty() {
                                    app.set_dst_text("".into());
                                    app.set_detected_lang_label("".into());
                                    app.set_is_loading(false);
                                }
                                show_app_window(&app);
                                position_window_at_cursor(&app);
                                app.invoke_focus_input();
                            }
                        }
                    });

                    // If just opened, refocus the input field for reliability
                    if !is_shown {
                        let app_w_delay = app_weak.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(Duration::from_millis(50));
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(app) = app_w_delay.upgrade() {
                                    app.invoke_focus_input();
                                }
                            });
                        });
                    }

                    continue;
                }

                // Capture selected text from active application
                let captured = capture_selected_text();

                // Only summon window if text was actually selected on screen!
                if captured.trim().is_empty() {
                    continue;
                }

                let app_weak_inner = app_weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = app_weak_inner.upgrade() {
                        let text = captured.clone();
                        app.set_src_text(text.clone().into());

                        let cur_src = app.get_src_lang().to_string();
                        let target_lang = if cur_src == "auto" {
                            let detected = auto_detect_target_lang(&text);
                            app.set_dst_lang(detected.into());
                            app.set_dst_lang_label(get_lang_label_for(detected, &app.get_ui_lang()).into());
                            let is_en = app.get_ui_lang() == "en";
                            let detected_src = if detected == "en" {
                                if is_en { "Cyrillic" } else { "Кирилиця" }
                            } else {
                                if is_en { "Latin" } else { "Латиниця" }
                            };
                            app.set_detected_lang_label(detected_src.into());
                            detected.to_string()
                        } else {
                            app.set_detected_lang_label("".into());
                            app.get_dst_lang().to_string()
                        };

                        app.set_status_text("".into());
                        show_app_window(&app);

                        position_window_at_cursor(&app);

                        app.invoke_focus_input();

                        // Launch translation
                        if !text.trim().is_empty() {
                            app.set_is_loading(true);
                            let app_weak_async = app.as_weak();
                            let s_lang = cur_src.clone();
                            let t_lang = target_lang.clone();
                            let deepl_key = app.get_deepl_key().to_string();
                            let use_deepl = app.get_use_deepl();

                            tokio::spawn(async move {
                                let res = translate_text(&text, &s_lang, &t_lang, &deepl_key, use_deepl).await;
                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(app) = app_weak_async.upgrade() {
                                        app.set_is_loading(false);
                                        match res {
                                            Ok((translated, detected_opt, engine_name, fell_back)) => {
                                                app.set_dst_text(translated.clone().into());
                                                app.set_engine_label(engine_name.into());
                                                if fell_back {
                                                    let is_en = app.get_ui_lang() == "en";
                                                    let msg = if is_en {
                                                        "⚠️ DeepL unavailable, switched to Google Translate".to_string()
                                                    } else {
                                                        "⚠️ DeepL недоступний, перекладено через Google Translate".to_string()
                                                    };
                                                    show_toast(app_weak_async.clone(), msg, "warning", 3500);
                                                }
                                                if s_lang == "auto" {
                                                    if let Some(detected) = detected_opt {
                                                        let label = get_lang_label_for(&detected, &app.get_ui_lang());
                                                        app.set_detected_lang_label(label.into());
                                                    }
                                                }
                                                if app.get_auto_copy() {
                                                    if let Ok(mut clip) = Clipboard::new() {
                                                        let _ = clip.set_text(translated);
                                                        let msg = if is_en { "Copied automatically! ✓" } else { "Скопійовано автоматично! ✓" };
                                                        set_status_timed(app_weak_async.clone(), msg.to_string());
                                                    }
                                                }
                                                if !fell_back && use_deepl && !deepl_key.trim().is_empty() {
                                                    update_deepl_quota_display(app_weak_async.clone(), deepl_key.clone());
                                                }
                                            }
                                            Err(err) => {
                                                let msg = if is_en { format!("Error: {}", err) } else { format!("Помилка: {}", err) };
                                                app.set_dst_text(msg.into());
                                            }
                                        }
                                    }
                                });
                            });
                        } else {
                            app.set_dst_text("".into());
                            app.set_detected_lang_label("".into());
                            app.set_is_loading(false);
                        }
                    }
                });
            }
        }
    });

    // 3. "Translate" handler (with 400ms debounce for typing)
    {
        let app_weak = app.as_weak();
        let debounce_clone = debounce_seq.clone();
        app.on_translate_requested(move || {
            if let Some(app) = app_weak.upgrade() {
                let text = app.get_src_text().to_string();
                if text.trim().is_empty() {
                    // Increment debounce seq to invalidate all background in-flight requests
                    let mut seq = debounce_clone.lock().unwrap();
                    *seq += 1;
                    app.set_dst_text("".into());
                    app.set_detected_lang_label("".into());
                    app.set_is_loading(false);
                    return;
                }

                let s_lang = app.get_src_lang().to_string();
                let t_lang = app.get_dst_lang().to_string();
                let deepl_key = app.get_deepl_key().to_string();
                let use_deepl = app.get_use_deepl();

                // Increment debounce counter
                let current_seq = {
                    let mut seq = debounce_clone.lock().unwrap();
                    *seq += 1;
                    *seq
                };

                app.set_is_loading(true);
                app.set_status_text("".into());

                let app_weak_async = app.as_weak();
                let debounce_wait = debounce_clone.clone();
                tokio::spawn(async move {
                    // Debounce: wait 400ms, then verify text hasn't changed
                    tokio::time::sleep(Duration::from_millis(400)).await;
                    let latest_seq = *debounce_wait.lock().unwrap();
                    if latest_seq != current_seq {
                        // Newer typing event arrived — cancel this request
                        return;
                    }

                    let res = translate_text(&text, &s_lang, &t_lang, &deepl_key, use_deepl).await;
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(app) = app_weak_async.upgrade() {
                            // If input was cleared in the meantime — do not set result
                            if app.get_src_text().trim().is_empty() {
                                app.set_dst_text("".into());
                                app.set_is_loading(false);
                                return;
                            }
                            // Verify sequence again: if seq doesn't match, drop result
                            let latest = *debounce_wait.lock().unwrap();
                            if latest != current_seq {
                                return;
                            }
                            app.set_is_loading(false);
                            match res {
                                Ok((translated, detected_opt, engine_name, fell_back)) => {
                                    app.set_dst_text(translated.clone().into());
                                    app.set_engine_label(engine_name.into());
                                    if fell_back {
                                        let is_en = app.get_ui_lang() == "en";
                                        let msg = if is_en {
                                            "⚠️ DeepL unavailable, switched to Google Translate".to_string()
                                        } else {
                                            "⚠️ DeepL недоступний, перекладено через Google Translate".to_string()
                                        };
                                        show_toast(app_weak_async.clone(), msg, "warning", 3500);
                                    }
                                    if s_lang == "auto" {
                                        if let Some(detected) = detected_opt {
                                            let label = get_lang_label_for(&detected, &app.get_ui_lang());
                                            app.set_detected_lang_label(label.into());
                                        }
                                    }
                                    if app.get_auto_copy() {
                                        if let Ok(mut clip) = Clipboard::new() {
                                            let _ = clip.set_text(translated);
                                            let msg = if is_en { "Copied automatically! ✓" } else { "Скопійовано автоматично! ✓" };
                                            set_status_timed(app_weak_async.clone(), msg.to_string());
                                        }
                                    }
                                    if !fell_back && use_deepl && !deepl_key.trim().is_empty() {
                                        update_deepl_quota_display(app_weak_async.clone(), deepl_key.clone());
                                    }
                                }
                                Err(err) => {
                                    let msg = if is_en { format!("Error: {}", err) } else { format!("Помилка: {}", err) };
                                    app.set_dst_text(msg.into());
                                }
                            }
                        }
                    });
                });
            }
        });
    }

    // Handler for shortcut settings changes
    {
        let config_clone = config.clone();
        let hotkey_mgr_clone = hotkey_manager.clone();
        let reg_keys_clone = registered_keys.clone();
        let app_weak = app.as_weak();
        app.on_shortcuts_changed(move |alt_c, alt_x| {
            let mut cfg = config_clone.lock().unwrap();
            cfg.enable_alt_c = alt_c;
            cfg.enable_alt_x = alt_x;
            if alt_c {
                cfg.custom_translate_shortcut.clear();
                cfg.enable_custom_translate = false;
                cfg.custom_shortcut.clear();
                cfg.enable_custom_shortcut = false;
            }
            if alt_x {
                cfg.custom_window_shortcut.clear();
                cfg.enable_custom_window = false;
                cfg.custom_shortcut.clear();
                cfg.enable_custom_shortcut = false;
            }
            cfg.save();
            let mgr = hotkey_mgr_clone.lock().unwrap();
            let mut reg = reg_keys_clone.lock().unwrap();
            sync_registered_hotkeys(&mgr, &mut reg, &cfg, app_weak.clone());
        });
    }

    // Handler for custom translate shortcut combination changes
    {
        let config_clone = config.clone();
        let hotkey_mgr_clone = hotkey_manager.clone();
        let reg_keys_clone = registered_keys.clone();
        let app_weak = app.as_weak();
        app.on_custom_translate_changed(move |combo| {
            let mut cfg = config_clone.lock().unwrap();
            let val = combo.to_string();
            cfg.custom_translate_shortcut = val.clone();
            if !val.trim().is_empty() {
                cfg.enable_custom_translate = true;
                cfg.enable_alt_c = false;
                cfg.custom_shortcut = val.clone();
                cfg.enable_custom_shortcut = true;
            } else {
                cfg.enable_custom_translate = false;
                cfg.enable_alt_c = true;
                cfg.custom_shortcut.clear();
                cfg.enable_custom_shortcut = false;
            }
            cfg.save();
            let mgr = hotkey_mgr_clone.lock().unwrap();
            let mut reg = reg_keys_clone.lock().unwrap();
            sync_registered_hotkeys(&mgr, &mut reg, &cfg, app_weak.clone());
        });
    }

    // Handler for toggling custom translate shortcut
    {
        let config_clone = config.clone();
        let hotkey_mgr_clone = hotkey_manager.clone();
        let reg_keys_clone = registered_keys.clone();
        let app_weak = app.as_weak();
        app.on_custom_translate_toggled(move |enabled| {
            let mut cfg = config_clone.lock().unwrap();
            cfg.enable_custom_translate = enabled;
            if enabled {
                cfg.enable_alt_c = false;
            }
            cfg.save();
            let mgr = hotkey_mgr_clone.lock().unwrap();
            let mut reg = reg_keys_clone.lock().unwrap();
            sync_registered_hotkeys(&mgr, &mut reg, &cfg, app_weak.clone());
        });
    }

    // Handler for custom window shortcut combination changes
    {
        let config_clone = config.clone();
        let hotkey_mgr_clone = hotkey_manager.clone();
        let reg_keys_clone = registered_keys.clone();
        let app_weak = app.as_weak();
        app.on_custom_window_changed(move |combo| {
            let mut cfg = config_clone.lock().unwrap();
            let val = combo.to_string();
            cfg.custom_window_shortcut = val.clone();
            if !val.trim().is_empty() {
                cfg.enable_custom_window = true;
                cfg.enable_alt_x = false;
                cfg.custom_shortcut = val.clone();
                cfg.enable_custom_shortcut = true;
            } else {
                cfg.enable_custom_window = false;
                cfg.enable_alt_x = true;
                cfg.custom_shortcut.clear();
                cfg.enable_custom_shortcut = false;
            }
            cfg.save();
            let mgr = hotkey_mgr_clone.lock().unwrap();
            let mut reg = reg_keys_clone.lock().unwrap();
            sync_registered_hotkeys(&mgr, &mut reg, &cfg, app_weak.clone());
        });
    }

    // Handler for toggling custom window shortcut
    {
        let config_clone = config.clone();
        let hotkey_mgr_clone = hotkey_manager.clone();
        let reg_keys_clone = registered_keys.clone();
        let app_weak = app.as_weak();
        app.on_custom_window_toggled(move |enabled| {
            let mut cfg = config_clone.lock().unwrap();
            cfg.enable_custom_window = enabled;
            if enabled {
                cfg.enable_alt_x = false;
            }
            cfg.save();
            let mgr = hotkey_mgr_clone.lock().unwrap();
            let mut reg = reg_keys_clone.lock().unwrap();
            sync_registered_hotkeys(&mgr, &mut reg, &cfg, app_weak.clone());
        });
    }

    // Handler for action tab selection changes
    {
        let config_clone = config.clone();
        app.on_custom_action_changed(move |action| {
            let mut cfg = config_clone.lock().unwrap();
            cfg.custom_action = action.to_string();
            cfg.save();
        });
    }

    app.on_custom_shortcut_changed(|_| {});
    app.on_custom_shortcut_toggled(|_| {});

    // Handler for key combination recording state changes (start / end)
    {
        let hotkey_mgr_clone = hotkey_manager.clone();
        let reg_keys_clone = registered_keys.clone();
        let config_clone = config.clone();
        let app_weak = app.as_weak();
        app.on_recording_state_changed(move |is_recording| {
            IS_RECORDING_SHORTCUT.store(is_recording, Ordering::Relaxed);
            let mgr = hotkey_mgr_clone.lock().unwrap();
            let mut reg = reg_keys_clone.lock().unwrap();
            if is_recording {
                // Temporarily unregister global hotkeys so recording field can receive keystrokes (including Alt+X, Alt+C etc.)
                for hk in reg.drain(..) {
                    let _ = mgr.unregister(hk);
                }
            } else {
                // Restore active hotkeys from current configuration
                let cfg = config_clone.lock().unwrap();
                sync_registered_hotkeys(&mgr, &mut reg, &cfg, app_weak.clone());
            }
        });
    }

    // Handler for Windows Autostart toggle
    {
        let config_clone = config.clone();
        let app_weak = app.as_weak();
        app.on_autostart_toggled(move |enabled| {
            let success = set_autostart(enabled);
            let mut cfg = config_clone.lock().unwrap();
            cfg.autostart = enabled;
            cfg.save();
            let ui_lang = cfg.ui_lang.clone();
            if success {
                let msg = if enabled {
                    if ui_lang == "en" { "Autostart with Windows enabled ✓" } else { "Автозавантаження з Windows увімкнено ✓" }
                } else {
                    if ui_lang == "en" { "Autostart with Windows disabled" } else { "Автозавантаження з Windows вимкнено" }
                };
                show_toast(app_weak.clone(), msg.to_string(), "success", 2000);
            }
        });
    }

    // Handler for dismissing notification Toast
    {
        let app_weak = app.as_weak();
        app.on_dismiss_toast(move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_show_toast(false);
            }
        });
    }

    // 4. "⇄" (Swap languages) handler
    {
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        app.on_swap_languages(move || {
            if let Some(app) = app_weak.upgrade() {
                let s = app.get_src_lang().to_string();
                let d = app.get_dst_lang().to_string();

                // On swap: new source = old target (never "auto")
                // new target = old source (or "uk" if source was "auto")
                let new_s = d.clone();
                let new_d = if s == "auto" { "uk".to_string() } else { s };

                app.set_src_lang(new_s.clone().into());
                app.set_dst_lang(new_d.clone().into());
                app.set_src_lang_label(get_lang_label_for(&new_s, &app.get_ui_lang()).into());
                app.set_dst_lang_label(get_lang_label_for(&new_d, &app.get_ui_lang()).into());
                app.set_detected_lang_label("".into()); // reset auto-detect label

                {
                    let mut cfg = config_clone.lock().unwrap();
                    // Save src_lang to config only if user previously selected a fixed language explicitly;
                    // do not overwrite "auto" if default auto-detection is active
                    if cfg.src_lang != "auto" {
                        cfg.src_lang = new_s;
                    }
                    cfg.dst_lang = new_d;
                    cfg.save();
                }

                let dst_text = app.get_dst_text().to_string();
                if !dst_text.is_empty() {
                    app.set_src_text(dst_text.into());
                    app.set_dst_text("".into());
                    app.invoke_translate_requested();
                }
            }
        });
    }

    // 5. Source and target language selection handlers
    {
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        app.on_select_src_lang(move |code| {
            if let Some(app) = app_weak.upgrade() {
                let code_str = code.to_string();
                app.set_src_lang(code_str.clone().into());
                app.set_src_lang_label(get_lang_label_for(&code_str, &app.get_ui_lang()).into());
                {
                    let mut cfg = config_clone.lock().unwrap();
                    cfg.src_lang = code_str;
                    cfg.save();
                }
                app.invoke_translate_requested();
            }
        });
    }
    {
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        app.on_select_dst_lang(move |code| {
            if let Some(app) = app_weak.upgrade() {
                let code_str = code.to_string();
                app.set_dst_lang(code_str.clone().into());
                app.set_dst_lang_label(get_lang_label_for(&code_str, &app.get_ui_lang()).into());
                {
                    let mut cfg = config_clone.lock().unwrap();
                    cfg.dst_lang = code_str;
                    cfg.save();
                }
                app.invoke_translate_requested();
            }
        });
    }

    // 6. Copy to clipboard handler
    {
        let app_weak = app.as_weak();
        app.on_copy_requested(move || {
            if let Some(app) = app_weak.upgrade() {
                let is_en = app.get_ui_lang() == "en";
                let text = app.get_dst_text().to_string();
                if !text.is_empty() {
                    if let Ok(mut clip) = Clipboard::new() {
                        let _ = clip.set_text(text);
                        let msg = if is_en { "Copied! ✓" } else { "Скопійовано! ✓" };
                        set_status_timed(app_weak.clone(), msg.to_string());
                    }
                } else {
                    let msg = if is_en { "No text to copy" } else { "Немає тексту для копіювання" };
                    set_status_timed(app_weak.clone(), msg.to_string());
                }
            }
        });
    }

    // 7. Paste back into previous window handler
    {
        let app_weak = app.as_weak();
        let prev_window_clone = prev_window.clone();
        app.on_paste_back_requested(move || {
            if let Some(app) = app_weak.upgrade() {
                let text = app.get_dst_text().to_string();
                if !text.is_empty() {
                    if let Ok(mut clip) = Clipboard::new() {
                        let _ = clip.set_text(text);
                    }
                    if app.get_close_on_paste() {
                        hide_app_window(&app);
                    }

                    // Focus previous window and send Ctrl+V
                    if let Some(raw_hwnd) = *prev_window_clone.lock().unwrap() {
                        unsafe {
                            let hwnd = HWND(raw_hwnd as *mut core::ffi::c_void);
                            let _ = SetForegroundWindow(hwnd);
                            std::thread::sleep(Duration::from_millis(50));
                            simulate_ctrl_key(VK_V);
                        }
                    }
                }
            }
        });
    }

    // 8. Hide to tray (close button × in titlebar)
    {
        let app_weak = app.as_weak();
        app.on_hide_to_tray(move || {
            if let Some(app) = app_weak.upgrade() {
                hide_app_window(&app);
            }
        });
    }

    // 8b. System window close request (Alt+F4 etc.)
    {
        app.window().on_close_requested(|| {
            slint::CloseRequestResponse::HideWindow
        });
    }



    // 9b. Context menu: Copy
    {
        let app_weak = app.as_weak();
        app.on_ctx_copy_requested(move |for_src| {
            if let Some(app) = app_weak.upgrade() {
                let text = if for_src {
                    app.get_src_text().to_string()
                } else {
                    app.get_dst_text().to_string()
                };
                if !text.is_empty() {
                    if let Ok(mut clip) = Clipboard::new() {
                        let _ = clip.set_text(text);
                        let is_en = app.get_ui_lang() == "en";
                        set_status_timed(
                            app_weak.clone(),
                            if is_en { "Copied! ✓".to_string() } else { "Скопійовано! ✓".to_string() },
                        );
                    }
                }
            }
        });
    }

    // 9c. Context menu: Paste
    {
        let app_weak = app.as_weak();
        app.on_ctx_paste_requested(move || {
            if let Some(app) = app_weak.upgrade() {
                if let Ok(mut clip) = Clipboard::new() {
                    if let Ok(text) = clip.get_text() {
                        if !text.is_empty() {
                            app.set_src_text(text.into());
                            app.invoke_translate_requested();
                            app.invoke_focus_input();
                            let is_en = app.get_ui_lang() == "en";
                            set_status_timed(
                                app_weak.clone(),
                                if is_en { "Pasted! ✓".to_string() } else { "Вставлено! ✓".to_string() },
                            );
                        }
                    }
                }
            }
        });
    }

    // 10. Smooth window dragging via absolute cursor coordinates at high refresh rate (180+ Hz)
    {
        let drag_state_clone = drag_state.clone();
        let app_weak = app.as_weak();
        app.on_start_window_drag(move || {
            if let Some(app) = app_weak.upgrade() {
                unsafe {
                    let mut pt = POINT { x: 0, y: 0 };
                    if GetCursorPos(&mut pt).is_ok() {
                        let pos = app.window().position();
                        *drag_state_clone.lock().unwrap() = Some((pt.x, pt.y, pos.x, pos.y));
                    }
                }
            }
        });
    }

    {
        let drag_state_clone = drag_state.clone();
        let app_weak = app.as_weak();
        app.on_window_drag_moved(move || {
            // Check physical state of left mouse button: if not pressed (0x8000), immediately cancel drag
            unsafe {
                if (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) == 0 {
                    *drag_state_clone.lock().unwrap() = None;
                    return;
                }
            }

            if let Some(app) = app_weak.upgrade() {
                unsafe {
                    let mut pt = POINT { x: 0, y: 0 };
                    if GetCursorPos(&mut pt).is_ok() {
                        if let Some((start_cx, start_cy, start_wx, start_wy)) = *drag_state_clone.lock().unwrap() {
                            let new_x = start_wx + (pt.x - start_cx);
                            let new_y = start_wy + (pt.y - start_cy);
                            app.window().set_position(slint::PhysicalPosition::new(new_x, new_y));
                        }
                    }
                }
            }
        });
    }

    {
        let drag_state_clone = drag_state.clone();
        let config_clone = config.clone();
        let app_weak = app.as_weak();
        app.on_end_window_drag(move || {
            *drag_state_clone.lock().unwrap() = None;
            // Persist new window coordinates
            if let Some(app) = app_weak.upgrade() {
                let pos = app.window().position();
                let mut cfg = config_clone.lock().unwrap();
                cfg.win_x = Some(pos.x);
                cfg.win_y = Some(pos.y);
                cfg.save();
            }
        });
    }

    // 11. Clear (text is cleared on Slint UI side, here only status and detected-lang)
    {
        let app_weak = app.as_weak();
        app.on_clear_requested(move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_status_text("".into());
                app.set_detected_lang_label("".into());
            }
        });
    }

    // 11b. Handler for auto-clearing status (from Slint)
    {
        let app_weak = app.as_weak();
        app.on_status_clear_requested(move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_status_text("".into());
            }
        });
    }

    // 12. Settings change (persist and apply window styles dynamically)
    {
        let config_clone = config.clone();
        app.on_settings_changed(move |popup_mode, pin_on_top, auto_copy, close_on_paste| {
            {
                let mut cfg = config_clone.lock().unwrap();
                cfg.popup_mode = popup_mode;
                cfg.always_on_top = pin_on_top;
                cfg.auto_copy = auto_copy;
                cfg.close_on_paste = close_on_paste;
                cfg.save();
            }
            apply_window_styles(popup_mode, pin_on_top);
        });
    }

    // 13. Priority languages toggling
    {
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        app.on_priority_lang_toggled(move |code| {
            if let Some(app) = app_weak.upgrade() {
                let code_str = code.to_string();
                let mut cfg = config_clone.lock().unwrap();
                if let Some(pos) = cfg.priority_langs.iter().position(|l| l == &code_str) {
                    // Already present — remove
                    cfg.priority_langs.remove(pos);
                } else if cfg.priority_langs.len() < 6 {
                    // Not present and slots available — add
                    cfg.priority_langs.push(code_str);
                }
                cfg.save();
                apply_priority_langs(&app, &cfg.priority_langs);
            }
        });
    }

    // 14. UI language change
    {
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        let show_item_clone = show_item.clone();
        let settings_item_clone = settings_item.clone();
        let quit_item_clone = quit_item.clone();
        app.on_ui_lang_changed(move |lang| {
            if let Some(app) = app_weak.upgrade() {
                let lang_str = lang.to_string();
                let (src_lang, dst_lang) = {
                    let mut cfg = config_clone.lock().unwrap();
                    cfg.ui_lang = lang_str.clone();
                    cfg.save();
                    (cfg.src_lang.clone(), cfg.dst_lang.clone())
                };
                // Update system tray menu items language
                if lang_str == "en" {
                    show_item_clone.set_text("Open Louise Translator");
                    settings_item_clone.set_text("Settings ⚙");
                    quit_item_clone.set_text("Exit");
                } else {
                    show_item_clone.set_text("Відкрити Louise Translator");
                    settings_item_clone.set_text("Налаштування ⚙");
                    quit_item_clone.set_text("Вихід");
                }
                // Update displayed language labels
                app.set_src_lang_label(get_lang_label_for(&src_lang, &lang_str).into());
                app.set_dst_lang_label(get_lang_label_for(&dst_lang, &lang_str).into());
                let cur_det = app.get_detected_lang_label().to_string();
                if !cur_det.is_empty() {
                    if cur_det == "Кирилиця" || cur_det == "Cyrillic" {
                        app.set_detected_lang_label(if lang_str == "en" { "Cyrillic".into() } else { "Кирилиця".into() });
                    } else if cur_det == "Латиниця" || cur_det == "Latin" {
                        app.set_detected_lang_label(if lang_str == "en" { "Latin".into() } else { "Латиниця".into() });
                    }
                }
            }
        });
    }

    // 15. DeepL API settings update
    {
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        app.on_deepl_settings_changed(move |use_deepl, key| {
            let key_str = key.to_string();
            {
                let mut cfg = config_clone.lock().unwrap();
                cfg.use_deepl = use_deepl;
                cfg.deepl_key = key_str.clone();
                cfg.save();
            }
            if let Some(app) = app_weak.upgrade() {
                let badge = if use_deepl && !key_str.trim().is_empty() {
                    "DeepL ⚡"
                } else {
                    "Google"
                };
                app.set_engine_label(badge.into());
            }
            if use_deepl && !key_str.trim().is_empty() {
                update_deepl_quota_display(app_weak.clone(), key_str);
            }
        });
    }

    // 16. Refresh DeepL quota
    {
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        app.on_refresh_deepl_quota(move || {
            let key = {
                let cfg = config_clone.lock().unwrap();
                cfg.deepl_key.clone()
            };
            update_deepl_quota_display(app_weak.clone(), key);
        });
    }

    let _autostart = std::env::args().any(|a| a == "--autostart" || a == "--tray");
    println!("Louise Translator (Rust + Slint) started!");
    println!("Window ready. Running discreetly in system tray or via hotkeys.");

    {
        let cfg = config.lock().unwrap();
        // If autostart was active, update the registry key (with --autostart flag)
        if cfg.autostart {
            set_autostart(true);
        }
        apply_window_styles(cfg.popup_mode, cfg.always_on_top);
        if let (Some(x), Some(y)) = (cfg.win_x, cfg.win_y) {
            app.window().set_position(slint::PhysicalPosition::new(x, y));
        }
    }

    // Start in background mode: window is hidden in system tray and waits for Alt+X / Alt+C or tray click
    log("Started in background tray mode: staying hidden in system tray until invoked.");

    log("Entering Slint event loop...");
    let event_res = slint::run_event_loop_until_quit();
    log(&format!("Slint event loop exited with: {:?}", event_res));

    event_res?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_detect_target_lang() {
        assert_eq!(auto_detect_target_lang("Привіт світ"), "en");
        assert_eq!(auto_detect_target_lang("Привет мир"), "en");
        assert_eq!(auto_detect_target_lang("Это тест"), "uk");
        assert_eq!(auto_detect_target_lang("Hello world"), "uk");
        assert_eq!(auto_detect_target_lang("Dzień dobry"), "uk");
    }

    #[test]
    fn test_lang_labels() {
        assert_eq!(get_lang_label("uk"), "Українська");
        assert_eq!(get_lang_label_en("uk"), "Ukrainian");
        assert_eq!(get_lang_label_for("en", "uk"), "Англійська");
        assert_eq!(get_lang_label_for("en", "en"), "English");
        assert_eq!(get_lang_label("auto"), "Автовизначення");
        assert_eq!(get_lang_label_en("auto"), "Auto-detect");
    }

    #[test]
    fn test_shortcut_parsing() {
        use global_hotkey::hotkey::{Code, Modifiers};
        let hk1 = parse_custom_hotkey("Alt+X").expect("valid Alt+X");
        assert_eq!(hk1.key, Code::KeyX);
        assert_eq!(hk1.mods, Modifiers::ALT);

        let hk2 = parse_custom_hotkey("Ctrl+Alt+C").expect("valid Ctrl+Alt+C");
        assert_eq!(hk2.key, Code::KeyC);
        assert_eq!(hk2.mods, Modifiers::CONTROL | Modifiers::ALT);
    }

    #[test]
    fn test_app_config_contract_and_defaults() {
        let json_data = r#"{
            "popup_mode": true,
            "always_on_top": true,
            "auto_copy": false,
            "close_on_paste": true,
            "src_lang": "auto",
            "dst_lang": "uk",
            "ui_lang": "en",
            "use_deepl": true,
            "deepl_key": "test_key"
        }"#;

        let cfg: AppConfig = serde_json::from_str(json_data).expect("valid config json");
        assert!(cfg.popup_mode);
        assert!(cfg.always_on_top);
        assert!(!cfg.auto_copy);
        assert!(cfg.close_on_paste);
        assert_eq!(cfg.src_lang, "auto");
        assert_eq!(cfg.dst_lang, "uk");
        assert_eq!(cfg.ui_lang, "en");
        assert!(cfg.use_deepl);
        assert_eq!(cfg.deepl_key, "test_key");

        let serialized = serde_json::to_string(&cfg).expect("serialize config");
        assert!(serialized.contains("\"popup_mode\":true"));
        assert!(serialized.contains("\"ui_lang\":\"en\""));
    }

    #[test]
    fn test_dpapi_roundtrip() {
        let original_key = "18469c54-7d68-44fd-82c7-a80373cf81b0:fx";
        let encrypted = config::dpapi_encrypt(original_key).expect("encryption succeeds");
        assert!(encrypted.starts_with("dpapi:"));
        assert_ne!(encrypted, original_key);
        let decrypted = config::dpapi_decrypt(&encrypted).expect("decryption succeeds");
        assert_eq!(decrypted, original_key);
    }

    #[test]
    fn test_config_dpapi_backward_compatibility() {
        // 1. Plaintext backwards compatibility
        let json_plain = r#"{"deepl_key": "legacy_plain_key"}"#;
        let mut cfg: AppConfig = serde_json::from_str(json_plain).unwrap();
        if cfg.deepl_key.starts_with("dpapi:") {
            cfg.deepl_key = config::dpapi_decrypt(&cfg.deepl_key).unwrap();
        }
        assert_eq!(cfg.deepl_key, "legacy_plain_key");

        // 2. Encrypted key loading
        let encrypted = config::dpapi_encrypt("legacy_plain_key").unwrap();
        let json_encrypted = format!(r#"{{"deepl_key": "{}"}}"#, encrypted);
        let mut cfg2: AppConfig = serde_json::from_str(&json_encrypted).unwrap();
        if cfg2.deepl_key.starts_with("dpapi:") {
            cfg2.deepl_key = config::dpapi_decrypt(&cfg2.deepl_key).unwrap();
        }
        assert_eq!(cfg2.deepl_key, "legacy_plain_key");
    }

    #[test]
    fn test_custom_action_config_defaults() {
        let default_cfg = AppConfig::default();
        assert_eq!(default_cfg.custom_action, "translate");

        let json_data = r#"{"custom_action": "toggle_window"}"#;
        let parsed: AppConfig = serde_json::from_str(json_data).unwrap();
        assert_eq!(parsed.custom_action, "toggle_window");

        let json_fallback = r#"{}"#;
        let parsed_fallback: AppConfig = serde_json::from_str(json_fallback).unwrap();
        assert_eq!(parsed_fallback.custom_action, "translate");
    }

    #[test]
    fn test_custom_shortcuts_distinct_fields_and_migration() {
        let default_cfg = AppConfig::default();
        assert_eq!(default_cfg.custom_translate_shortcut, "");
        assert_eq!(default_cfg.custom_window_shortcut, "");
        assert!(!default_cfg.enable_custom_translate);
        assert!(!default_cfg.enable_custom_window);

        let json_distinct = r#"{
            "custom_translate_shortcut": "Alt+Shift+Z",
            "enable_custom_translate": true,
            "custom_window_shortcut": "Alt+Q",
            "enable_custom_window": true
        }"#;
        let cfg: AppConfig = serde_json::from_str(json_distinct).unwrap();
        assert_eq!(cfg.custom_translate_shortcut, "Alt+Shift+Z");
        assert!(cfg.enable_custom_translate);
        assert_eq!(cfg.custom_window_shortcut, "Alt+Q");
        assert!(cfg.enable_custom_window);
    }

    #[test]
    fn test_load_specific_config_file() {
        if let Ok(content) = std::fs::read_to_string("config.json") {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                if cfg.deepl_key.starts_with("dpapi:") {
                    let res = config::dpapi_decrypt(&cfg.deepl_key);
                    assert!(res.is_ok());
                }
            }
        }
    }
}

