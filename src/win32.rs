use std::os::windows::process::CommandExt;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering};
use std::time::Duration;
use arboard::Clipboard;
use slint::ComponentHandle;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::core::{BOOL, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::Shell::{ITaskbarList, TaskbarList};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_C,
    VK_CONTROL, VK_MENU,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, EnumWindows, GetCursorPos, GetSystemMetrics,
    GetWindowLongPtrW, GetWindowLongW, GetWindowThreadProcessId, IsWindowVisible, RegisterClassW,
    SetForegroundWindow, SetWindowLongPtrW, SetWindowLongW, SetWindowPos, ShowWindow,
    GWLP_HWNDPARENT, GWL_EXSTYLE, GWL_STYLE, HWND_NOTOPMOST, HWND_TOPMOST,
    SM_CXSCREEN, SM_CYSCREEN, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SW_HIDE, WINDOW_EX_STYLE, WNDCLASSW, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW, WS_POPUP, WS_SYSMENU,
};
use crate::AppWindow;

pub static MAIN_THREAD_ID: AtomicU32 = AtomicU32::new(0);
pub static CACHED_HWND: AtomicIsize = AtomicIsize::new(0);
pub static POPUP_MODE: AtomicBool = AtomicBool::new(false);
pub static ALWAYS_ON_TOP: AtomicBool = AtomicBool::new(false);
pub static HIDDEN_OWNER_HWND: AtomicIsize = AtomicIsize::new(0);
pub static WINDOW_SHOWN: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn owner_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn get_or_create_hidden_owner() -> HWND {
    let raw = HIDDEN_OWNER_HWND.load(Ordering::Relaxed);
    if raw != 0 {
        return HWND(raw as *mut core::ffi::c_void);
    }
    unsafe {
        let hinst = GetModuleHandleW(None).unwrap_or_default();
        let class_name: Vec<u16> = "Louise.HiddenOwner\0".encode_utf16().collect();
        let win_name: Vec<u16> = "LouiseOwner\0".encode_utf16().collect();
        let wc = WNDCLASSW {
            lpfnWndProc: Some(owner_wnd_proc),
            hInstance: hinst.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        let _ = RegisterClassW(&wc);
        if let Ok(owner) = CreateWindowExW(
            WINDOW_EX_STYLE(WS_EX_TOOLWINDOW.0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(win_name.as_ptr()),
            WS_POPUP,
            0, 0, 0, 0,
            None,
            None,
            Some(hinst.into()),
            None,
        ) {
            HIDDEN_OWNER_HWND.store(owner.0 as isize, Ordering::Relaxed);
            log_diag(&format!("Created hidden owner window: {:?}", owner));
            return owner;
        }
    }
    HWND(core::ptr::null_mut())
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
    if pid == unsafe { GetCurrentProcessId() } {
        let raw_owner = HIDDEN_OWNER_HWND.load(Ordering::Relaxed);
        if raw_owner != 0 && hwnd.0 as isize == raw_owner {
            return BOOL(1);
        }
        let target = lparam.0 as *mut HWND;
        unsafe { *target = hwnd; }
        return BOOL(0);
    }
    BOOL(1)
}

pub fn get_app_hwnd() -> HWND {
    let raw = CACHED_HWND.load(Ordering::Relaxed);
    if raw != 0 {
        return HWND(raw as *mut core::ffi::c_void);
    }
    let mut found = HWND(core::ptr::null_mut());
    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut found as *mut HWND as isize),
        );
    }
    if !found.0.is_null() {
        CACHED_HWND.store(found.0 as isize, Ordering::Relaxed);
        log_diag(&format!("get_app_hwnd: Found HWND by PID/Title: {:?}", found));
        return found;
    }
    HWND(core::ptr::null_mut())
}

pub fn log_diag(msg: &str) {
    use std::io::Write;
    let path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("diag.log")))
        .unwrap_or_else(|| std::path::PathBuf::from("diag.log"));
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "[{}] {}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis(), msg);
        let _ = f.flush();
    }
}

pub fn cache_hwnd_from_window(w: &slint::Window) -> HWND {
    if let Ok(handle) = w.window_handle().window_handle() {
        if let RawWindowHandle::Win32(win32) = handle.as_raw() {
            let hwnd = HWND(win32.hwnd.get() as *mut core::ffi::c_void);
            CACHED_HWND.store(win32.hwnd.get() as isize, Ordering::Relaxed);
            log_diag(&format!("cache_hwnd_from_window SUCCESS: hwnd={:?}", hwnd));
            return hwnd;
        }
    }
    get_app_hwnd()
}

pub unsafe fn set_ex(hwnd: HWND, add: u32, remove: u32) {
    unsafe {
        let cur = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let new = (cur & !remove) | add;
        let res = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new as isize);
        let after = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        log_diag(&format!("set_ex: cur=0x{:X}, target=0x{:X}, res=0x{:X}, after=0x{:X}", cur, new, res, after));
    }
}

pub fn update_taskbar_presence(hwnd: HWND, popup_mode: bool) {
    log_diag(&format!("update_taskbar_presence: hwnd={:?}, popup_mode={}", hwnd, popup_mode));
    if hwnd.0.is_null() {
        return;
    }
    unsafe {
        if popup_mode {
            let owner = get_or_create_hidden_owner();
            if !owner.0.is_null() {
                let _ = SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, owner.0 as isize);
            }
            set_ex(hwnd, WS_EX_TOOLWINDOW.0, WS_EX_APPWINDOW.0);
            match CoCreateInstance::<_, ITaskbarList>(&TaskbarList, None, CLSCTX_INPROC_SERVER) {
                Ok(taskbar) => {
                    let hr_init = taskbar.HrInit();
                    let hr_del = taskbar.DeleteTab(hwnd);
                    log_diag(&format!("TaskbarList: HrInit={:?}, DeleteTab={:?}", hr_init, hr_del));
                }
                Err(err) => {
                    log_diag(&format!("TaskbarList: CoCreateInstance FAILED: {:?}", err));
                }
            }
        } else {
            let _ = SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, 0);
            set_ex(hwnd, WS_EX_APPWINDOW.0, WS_EX_TOOLWINDOW.0);
            if let Ok(taskbar) = CoCreateInstance::<_, ITaskbarList>(&TaskbarList, None, CLSCTX_INPROC_SERVER) {
                let _ = taskbar.HrInit();
                let _ = taskbar.AddTab(hwnd);
            }
        }
        let swp_res = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
        log_diag(&format!("SetWindowPos SWP_FRAMECHANGED: {:?}", swp_res));
    }
}

pub fn apply_window_styles(popup_mode: bool, always_on_top: bool) {
    POPUP_MODE.store(popup_mode, Ordering::Relaxed);
    ALWAYS_ON_TOP.store(always_on_top, Ordering::Relaxed);
    let hwnd = get_app_hwnd();
    if !hwnd.0.is_null() {
        unsafe {
            let mut style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            style |= WS_SYSMENU.0;
            SetWindowLongW(hwnd, GWL_STYLE, style as i32);

            update_taskbar_presence(hwnd, popup_mode);

            let topmost = if always_on_top {
                HWND_TOPMOST
            } else {
                HWND_NOTOPMOST
            };
            let _ = SetWindowPos(
                hwnd,
                Some(topmost),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
        }
    }
}

pub fn is_window_shown() -> bool {
    let raw = CACHED_HWND.load(Ordering::Relaxed);
    if raw != 0 {
        let hwnd = HWND(raw as *mut core::ffi::c_void);
        if unsafe { IsWindowVisible(hwnd).as_bool() } {
            return true;
        }
    }
    WINDOW_SHOWN.load(Ordering::Relaxed)
}

pub fn show_app_window(app: &AppWindow) {
    WINDOW_SHOWN.store(true, Ordering::Relaxed);
    let _ = app.show();
    let hwnd = get_app_hwnd();
    let on_top = ALWAYS_ON_TOP.load(Ordering::Relaxed);

    if !hwnd.0.is_null() {
        unsafe {
            let topmost = if on_top { HWND_TOPMOST } else { HWND_NOTOPMOST };
            let _ = SetWindowPos(
                hwnd,
                Some(topmost),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

pub fn hide_app_window(app: &AppWindow) {
    WINDOW_SHOWN.store(false, Ordering::Relaxed);
    let hwnd = cache_hwnd_from_window(app.window());
    if !hwnd.0.is_null() {
        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
    }
    let _ = app.hide();
}

// Positions the window so mouse cursor is centered on the SOURCE text input box
pub fn position_window_at_cursor(app: &AppWindow) {
    unsafe {
        let mut pt = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut pt).is_ok() {
            let win_w = 500;
            let win_h = 540;

            // Center of the input field inside the window (500x540):
            // X center = 250px (half of 500px width)
            // Y center = padding (10) + header (32) + spacing (6) + selector (38) + spacing (6) + source_label (16) + spacing (4) + (box_height 174 / 2) = 199px ≈ 200px
            let input_center_x = 250;
            let input_center_y = 200;

            let mut x = pt.x - input_center_x;
            let mut y = pt.y - input_center_y;

            let hmonitor = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                rcMonitor: Default::default(),
                rcWork: Default::default(),
                dwFlags: 0,
            };

            if GetMonitorInfoW(hmonitor, &mut mi).as_bool() {
                let margin = 10;
                let min_x = mi.rcWork.left + margin;
                let max_x = (mi.rcWork.right - win_w - margin).max(min_x);
                let min_y = mi.rcWork.top + margin;
                let max_y = (mi.rcWork.bottom - win_h - margin).max(min_y);

                x = x.clamp(min_x, max_x);
                y = y.clamp(min_y, max_y);
            } else {
                let screen_w = GetSystemMetrics(SM_CXSCREEN);
                let screen_h = GetSystemMetrics(SM_CYSCREEN);
                x = x.clamp(10, (screen_w - win_w - 10).max(10));
                y = y.clamp(10, (screen_h - win_h - 10).max(10));
            }

            app.window().set_position(slint::PhysicalPosition::new(x, y));
        }
    }
}

// Adds or removes the application from Windows Startup (HKCU\...\Run)
pub fn set_autostart(enable: bool) -> bool {
    let exe_path = match std::env::current_exe() {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => return false,
    };
    if enable {
        let val = format!("\"{}\" --autostart", exe_path);
        let status = std::process::Command::new("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "Louise Translator",
                "/t",
                "REG_SZ",
                "/d",
                &val,
                "/f",
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .status();
        status.map(|s| s.success()).unwrap_or(false)
    } else {
        let status = std::process::Command::new("reg")
            .args(&[
                "delete",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "Louise Translator",
                "/f",
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .status();
        status.map(|s| s.success()).unwrap_or(false)
    }
}

// Checks if autostart is enabled in the registry
pub fn is_autostart_enabled() -> bool {
    let output = std::process::Command::new("reg")
        .args(&[
            "query",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v",
            "Louise Translator",
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();
    output.map(|o| o.status.success()).unwrap_or(false)
}

// Safe key press emulation via Win32 SendInput
pub unsafe fn simulate_ctrl_key(key: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY) {
    // 1. Release Alt
    let release_alt = [INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VK_MENU,
                wScan: 0,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }];
    let _ = unsafe { SendInput(&release_alt, std::mem::size_of::<INPUT>() as i32) };
    std::thread::sleep(Duration::from_millis(25));

    // 2. Ctrl + Key Down
    let inputs_down = [
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
                    wVk: key,
                    wScan: 0,
                    dwFlags: Default::default(),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];
    let _ = unsafe { SendInput(&inputs_down, std::mem::size_of::<INPUT>() as i32) };

    std::thread::sleep(Duration::from_millis(30));

    // 3. Key + Ctrl Up
    let inputs_up = [
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
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
    let _ = unsafe { SendInput(&inputs_up, std::mem::size_of::<INPUT>() as i32) };
}

// Captures selected text via system clipboard
pub fn capture_selected_text() -> String {
    unsafe {
        // Prevent triggering the Alt menu bar in Chrome/browsers
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
        std::thread::sleep(Duration::from_millis(35));

        let mut clip = match Clipboard::new() {
            Ok(c) => c,
            Err(_) => return String::new(),
        };

        let backup = clip.get_text().unwrap_or_default();

        // Emulate Ctrl+C
        simulate_ctrl_key(VK_C);

        // Wait for clipboard update
        for _ in 0..25 {
            std::thread::sleep(Duration::from_millis(15));
            if let Ok(new_text) = clip.get_text() {
                if new_text != backup && !new_text.trim().is_empty() {
                    return new_text;
                }
            }
        }

        clip.get_text().unwrap_or_default()
    }
}
