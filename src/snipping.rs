use std::sync::atomic::AtomicBool;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush, DeleteDC,
    DeleteObject, EndPaint, FrameRect, InvalidateRect, SelectObject, SetDIBits, UpdateWindow,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC, PAINTSTRUCT, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, SetFocus, VK_ESCAPE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetSystemMetrics, LoadCursorW, PostQuitMessage, RegisterClassW, SetCursor,
    SetForegroundWindow, ShowWindow, TranslateMessage, GetWindowLongPtrW, SetWindowLongPtrW,
    GWLP_USERDATA, IDC_CROSS, MSG, SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYSCREEN,
    SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SW_SHOW, WINDOW_EX_STYLE,
    WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT,
    WM_RBUTTONDOWN, WM_SETCURSOR, WNDCLASSW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    WS_VISIBLE,
};

pub static SNIPPING_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn is_snipping_active() -> bool {
    SNIPPING_ACTIVE.load(std::sync::atomic::Ordering::SeqCst)
}

#[derive(Debug, Clone)]
pub struct SnipCapture {
    pub bgra: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub screen_x: i32,
    pub screen_y: i32,
}

/// Normalizes two mouse coordinates into `(left, top, width, height)` clamped to `[0..max_w, 0..max_h]`.
pub fn normalize_selection_rect(
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    max_w: i32,
    max_h: i32,
) -> (i32, i32, i32, i32) {
    let left = x1.min(x2).clamp(0, max_w.max(0));
    let top = y1.min(y2).clamp(0, max_h.max(0));
    let right = x1.max(x2).clamp(0, max_w.max(0));
    let bottom = y1.max(y2).clamp(0, max_h.max(0));
    (left, top, (right - left).max(0), (bottom - top).max(0))
}

/// Creates a darkened copy of a BGRA8 buffer (approx. 45% brightness) for the snipping backdrop.
pub fn create_dimmed_bgra(src: &[u8]) -> Vec<u8> {
    let mut dimmed = vec![0u8; src.len()];
    for (dst_px, src_px) in dimmed.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
        dst_px[0] = ((src_px[0] as u16 * 115) >> 8) as u8;
        dst_px[1] = ((src_px[1] as u16 * 115) >> 8) as u8;
        dst_px[2] = ((src_px[2] as u16 * 115) >> 8) as u8;
        dst_px[3] = 255;
    }
    dimmed
}

/// Crops a rectangular sub-region `(crop_x, crop_y, crop_w, crop_h)` from a top-down BGRA8 image.
pub fn crop_bgra_buffer(
    src: &[u8],
    src_w: u32,
    src_h: u32,
    crop_x: u32,
    crop_y: u32,
    crop_w: u32,
    crop_h: u32,
) -> Vec<u8> {
    if crop_w == 0 || crop_h == 0 || src_w == 0 || src_h == 0 {
        return Vec::new();
    }
    let clamped_w = crop_w.min(src_w.saturating_sub(crop_x));
    let clamped_h = crop_h.min(src_h.saturating_sub(crop_y));
    if clamped_w == 0 || clamped_h == 0 {
        return Vec::new();
    }

    let mut out = vec![0u8; (clamped_w as usize) * (clamped_h as usize) * 4];
    for row in 0..clamped_h {
        let sy = (crop_y + row) as usize;
        let sx = crop_x as usize;
        let src_start = (sy * (src_w as usize) + sx) * 4;
        let src_end = src_start + (clamped_w as usize) * 4;
        let dst_start = (row as usize) * (clamped_w as usize) * 4;
        let dst_end = dst_start + (clamped_w as usize) * 4;
        out[dst_start..dst_end].copy_from_slice(&src[src_start..src_end]);
    }
    out
}

struct OverlayState {
    screen_w: i32,
    screen_h: i32,
    hdc_orig: HDC,
    hdc_dim: HDC,
    hdc_back: HDC,
    is_dragging: bool,
    start_x: i32,
    start_y: i32,
    curr_x: i32,
    curr_y: i32,
    confirmed_rect: Option<(i32, i32, i32, i32)>,
}

#[inline]
fn loword_i16(l: isize) -> i32 {
    (l & 0xFFFF) as i16 as i32
}

#[inline]
fn hiword_i16(l: isize) -> i32 {
    ((l >> 16) & 0xFFFF) as i16 as i32
}

unsafe extern "system" fn snip_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayState;

        match msg {
            WM_ERASEBKGND => LRESULT(1),
            WM_SETCURSOR => {
                if let Ok(hcur) = LoadCursorW(None, IDC_CROSS) {
                    SetCursor(Some(hcur));
                }
                LRESULT(1)
            }
            WM_KEYDOWN => {
                if wparam.0 == VK_ESCAPE.0 as usize {
                    if !state_ptr.is_null() {
                        (*state_ptr).confirmed_rect = None;
                        (*state_ptr).is_dragging = false;
                    }
                    let _ = ReleaseCapture();
                    let _ = DestroyWindow(hwnd);
                    return LRESULT(0);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_RBUTTONDOWN => {
                if !state_ptr.is_null() {
                    (*state_ptr).confirmed_rect = None;
                    (*state_ptr).is_dragging = false;
                }
                let _ = ReleaseCapture();
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                if !state_ptr.is_null() {
                    let st = &mut *state_ptr;
                    let x = loword_i16(lparam.0);
                    let y = hiword_i16(lparam.0);
                    st.is_dragging = true;
                    st.start_x = x;
                    st.start_y = y;
                    st.curr_x = x;
                    st.curr_y = y;
                    SetCapture(hwnd);
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                if !state_ptr.is_null() {
                    let st = &mut *state_ptr;
                    if st.is_dragging {
                        st.curr_x = loword_i16(lparam.0);
                        st.curr_y = hiword_i16(lparam.0);
                        let _ = InvalidateRect(Some(hwnd), None, false);
                        let _ = UpdateWindow(hwnd);
                    }
                }
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                if !state_ptr.is_null() {
                    let st = &mut *state_ptr;
                    if st.is_dragging {
                        st.is_dragging = false;
                        st.curr_x = loword_i16(lparam.0);
                        st.curr_y = hiword_i16(lparam.0);
                        let _ = ReleaseCapture();

                        let (rx, ry, rw, rh) = normalize_selection_rect(
                            st.start_x,
                            st.start_y,
                            st.curr_x,
                            st.curr_y,
                            st.screen_w,
                            st.screen_h,
                        );
                        if rw >= 8 && rh >= 8 {
                            st.confirmed_rect = Some((rx, ry, rw, rh));
                        } else {
                            st.confirmed_rect = None;
                        }
                        let _ = DestroyWindow(hwnd);
                    }
                }
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                if !state_ptr.is_null() && !hdc.is_invalid() {
                    let st = &*state_ptr;
                    // 1. Copy dimmed screen to backbuffer
                    let _ = BitBlt(
                        st.hdc_back,
                        0,
                        0,
                        st.screen_w,
                        st.screen_h,
                        Some(st.hdc_dim),
                        0,
                        0,
                        SRCCOPY,
                    );

                    // 2. If dragging, draw bright original region & monochrome border
                    if st.is_dragging {
                        let (rx, ry, rw, rh) = normalize_selection_rect(
                            st.start_x,
                            st.start_y,
                            st.curr_x,
                            st.curr_y,
                            st.screen_w,
                            st.screen_h,
                        );
                        if rw > 0 && rh > 0 {
                            let _ = BitBlt(
                                st.hdc_back,
                                rx,
                                ry,
                                rw,
                                rh,
                                Some(st.hdc_orig),
                                rx,
                                ry,
                                SRCCOPY,
                            );

                            // 2px crisp border in #f1ffff (COLORREF = 0x00BBGGRR = 0x00FFFFF1)
                            let border_brush = CreateSolidBrush(COLORREF(0x00FFFFF1));
                            let outer_rc = RECT {
                                left: (rx - 1).max(0),
                                top: (ry - 1).max(0),
                                right: (rx + rw + 1).min(st.screen_w),
                                bottom: (ry + rh + 1).min(st.screen_h),
                            };
                            let inner_rc = RECT {
                                left: rx,
                                top: ry,
                                right: rx + rw,
                                bottom: ry + rh,
                            };
                            FrameRect(st.hdc_back, &outer_rc, border_brush);
                            FrameRect(st.hdc_back, &inner_rc, border_brush);
                            let _ = DeleteObject(border_brush.into());
                        }
                    }

                    // 3. Present backbuffer to screen window
                    let _ = BitBlt(
                        hdc,
                        0,
                        0,
                        st.screen_w,
                        st.screen_h,
                        Some(st.hdc_back),
                        0,
                        0,
                        SRCCOPY,
                    );
                }
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn create_bitmap_from_bgra(hdc_screen: HDC, width: i32, height: i32, pixels: &[u8]) -> HBITMAP {
    unsafe {
        let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbm.is_invalid() {
            return hbm;
        }
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        SetDIBits(
            Some(hdc_screen),
            hbm,
            0,
            height as u32,
            pixels.as_ptr() as *const core::ffi::c_void,
            &bmi,
            DIB_RGB_COLORS,
        );
        hbm
    }
}

/// Freezes the current virtual screen, shows a full-screen interactive crosshair selector,
/// and returns the cropped BGRA8 pixels of the user's selected rectangle (or `None` if cancelled).
pub fn run_snipping_overlay() -> Result<Option<SnipCapture>, String> {
    unsafe {
        let mut vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let mut vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let mut vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let mut vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        if vw <= 0 || vh <= 0 {
            vx = 0;
            vy = 0;
            vw = GetSystemMetrics(SM_CXSCREEN);
            vh = GetSystemMetrics(SM_CYSCREEN);
        }
        if vw <= 0 || vh <= 0 {
            return Err("Failed to query screen metrics".to_string());
        }

        // 1. Snapshot the screen BEFORE showing the overlay
        let orig_pixels = crate::ocr::capture_screen_region(vx, vy, vw as u32, vh as u32)?;
        let dim_pixels = create_dimmed_bgra(&orig_pixels);

        let display_name: Vec<u16> = "DISPLAY\0".encode_utf16().collect();
        let hdc_screen = windows::Win32::Graphics::Gdi::CreateDCW(
            PCWSTR(display_name.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            None,
        );
        if hdc_screen.is_invalid() {
            return Err("CreateDCW(DISPLAY) failed in overlay".to_string());
        }

        let hdc_orig = CreateCompatibleDC(Some(hdc_screen));
        let hdc_dim = CreateCompatibleDC(Some(hdc_screen));
        let hdc_back = CreateCompatibleDC(Some(hdc_screen));

        let hbm_orig = create_bitmap_from_bgra(hdc_screen, vw, vh, &orig_pixels);
        let hbm_dim = create_bitmap_from_bgra(hdc_screen, vw, vh, &dim_pixels);
        let hbm_back = CreateCompatibleBitmap(hdc_screen, vw, vh);

        let old_orig = SelectObject(hdc_orig, hbm_orig.into());
        let old_dim = SelectObject(hdc_dim, hbm_dim.into());
        let old_back = SelectObject(hdc_back, hbm_back.into());

        // Initialize backbuffer with dimmed screen
        let _ = BitBlt(hdc_back, 0, 0, vw, vh, Some(hdc_dim), 0, 0, SRCCOPY);

        let mut overlay_state = Box::new(OverlayState {
            screen_w: vw,
            screen_h: vh,
            hdc_orig,
            hdc_dim,
            hdc_back,
            is_dragging: false,
            start_x: 0,
            start_y: 0,
            curr_x: 0,
            curr_y: 0,
            confirmed_rect: None,
        });

        let hinst = GetModuleHandleW(None).unwrap_or_default();
        let class_name: Vec<u16> = "Louise.SnippingOverlay\0".encode_utf16().collect();
        let win_title: Vec<u16> = "Louise OCR Snip\0".encode_utf16().collect();

        let wc = WNDCLASSW {
            lpfnWndProc: Some(snip_wnd_proc),
            hInstance: hinst.into(),
            hCursor: LoadCursorW(None, IDC_CROSS).unwrap_or_default(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        let _ = RegisterClassW(&wc);

        let hwnd_res = CreateWindowExW(
            WINDOW_EX_STYLE(WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(win_title.as_ptr()),
            WS_POPUP | WS_VISIBLE,
            vx,
            vy,
            vw,
            vh,
            None,
            None,
            Some(hinst.into()),
            None,
        );

        let hwnd = match hwnd_res {
            Ok(h) => h,
            Err(e) => {
                SelectObject(hdc_orig, old_orig);
                SelectObject(hdc_dim, old_dim);
                SelectObject(hdc_back, old_back);
                let _ = DeleteObject(hbm_orig.into());
                let _ = DeleteObject(hbm_dim.into());
                let _ = DeleteObject(hbm_back.into());
                let _ = DeleteDC(hdc_orig);
                let _ = DeleteDC(hdc_dim);
                let _ = DeleteDC(hdc_back);
                let _ = DeleteDC(hdc_screen);
                return Err(format!("CreateWindowExW failed: {}", e));
            }
        };

        SetWindowLongPtrW(
            hwnd,
            GWLP_USERDATA,
            (&mut *overlay_state as *mut OverlayState) as isize,
        );

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(Some(hwnd));
        let _ = InvalidateRect(Some(hwnd), None, false);
        let _ = UpdateWindow(hwnd);

        // Modal message loop on this thread until WM_DESTROY -> PostQuitMessage(0)
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup GDI resources
        SelectObject(hdc_orig, old_orig);
        SelectObject(hdc_dim, old_dim);
        SelectObject(hdc_back, old_back);
        let _ = DeleteObject(hbm_orig.into());
        let _ = DeleteObject(hbm_dim.into());
        let _ = DeleteObject(hbm_back.into());
        let _ = DeleteDC(hdc_orig);
        let _ = DeleteDC(hdc_dim);
        let _ = DeleteDC(hdc_back);
        let _ = DeleteDC(hdc_screen);

        if let Some((rx, ry, rw, rh)) = overlay_state.confirmed_rect {
            let cropped = crop_bgra_buffer(
                &orig_pixels,
                vw as u32,
                vh as u32,
                rx as u32,
                ry as u32,
                rw as u32,
                rh as u32,
            );
            if cropped.is_empty() {
                return Ok(None);
            }
            return Ok(Some(SnipCapture {
                bgra: cropped,
                width: rw as u32,
                height: rh as u32,
                screen_x: vx + rx,
                screen_y: vy + ry,
            }));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_selection_rect() {
        // Normal top-left to bottom-right drag
        assert_eq!(
            normalize_selection_rect(10, 20, 110, 70, 1920, 1080),
            (10, 20, 100, 50)
        );
        // Reverse bottom-right to top-left drag
        assert_eq!(
            normalize_selection_rect(110, 70, 10, 20, 1920, 1080),
            (10, 20, 100, 50)
        );
        // Out-of-bounds clamping
        assert_eq!(
            normalize_selection_rect(-50, -20, 200, 100, 150, 80),
            (0, 0, 150, 80)
        );
    }

    #[test]
    fn test_create_dimmed_bgra() {
        let src = vec![200u8, 100u8, 50u8, 255u8];
        let dim = create_dimmed_bgra(&src);
        assert_eq!(dim.len(), 4);
        assert!(dim[0] < 200 && dim[0] > 80);
        assert!(dim[1] < 100 && dim[1] > 40);
        assert!(dim[2] < 50 && dim[2] > 20);
        assert_eq!(dim[3], 255);
    }

    #[test]
    fn test_crop_bgra_buffer() {
        // 4x4 image where each pixel (x, y) has B=x, G=y, R=10, A=255
        let mut src = Vec::with_capacity(4 * 4 * 4);
        for y in 0..4u8 {
            for x in 0..4u8 {
                src.extend_from_slice(&[x, y, 10, 255]);
            }
        }
        // Crop 2x2 starting at (1, 2)
        let cropped = crop_bgra_buffer(&src, 4, 4, 1, 2, 2, 2);
        assert_eq!(cropped.len(), 2 * 2 * 4);
        // Row 0 of crop: (1, 2) and (2, 2)
        assert_eq!(&cropped[0..4], &[1, 2, 10, 255]);
        assert_eq!(&cropped[4..8], &[2, 2, 10, 255]);
        // Row 1 of crop: (1, 3) and (2, 3)
        assert_eq!(&cropped[8..12], &[1, 3, 10, 255]);
        assert_eq!(&cropped[12..16], &[2, 3, 10, 255]);
    }
}
