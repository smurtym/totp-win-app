// Windows UI module
// Direct Win32 API bindings for native Windows controls

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::Controls::DRAWITEMSTRUCT,
    Win32::UI::WindowsAndMessaging::*,
};

use crate::totp::Account;
use std::cell::RefCell;
use std::time::{SystemTime, UNIX_EPOCH};

/// Window class name
const WINDOW_CLASS: &str = "TOTP_Window";

/// Timer IDs
const TIMER_UPDATE: usize = 1;

/// Control IDs
const ID_COPY_BUTTON_BASE: i32 = 2000;

/// Window state
pub struct WindowState {
    pub accounts: Vec<Account>,
    pub invalid_line_count: usize,
}

thread_local! {
    static WINDOW_STATE: RefCell<Option<WindowState>> = RefCell::new(None);
    static DARK_BRUSH:         RefCell<HBRUSH> = RefCell::new(HBRUSH(0));
    static BTN_NORMAL_BRUSH:   RefCell<HBRUSH> = RefCell::new(HBRUSH(0));
    static BTN_PRESSED_BRUSH:  RefCell<HBRUSH> = RefCell::new(HBRUSH(0));
    static BTN_BORDER_BRUSH:   RefCell<HBRUSH> = RefCell::new(HBRUSH(0));
    // (step, codes) — recomputed only when the 30s period rolls over
    static CODE_CACHE: RefCell<(u64, Vec<String>)> = RefCell::new((0, Vec::new()));
}

/// Initialize window state
pub fn init_window_state(accounts: Vec<Account>, invalid_line_count: usize) {
    WINDOW_STATE.with(|state| {
        *state.borrow_mut() = Some(WindowState {
            accounts,
            invalid_line_count,
        });
    });
}

fn with_window_state<F: FnOnce(&WindowState) -> R, R>(f: F) -> Option<R> {
    WINDOW_STATE.with(|s| s.borrow().as_ref().map(f))
}

/// Current TOTP 30s period index
fn current_step() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() / 30
}

/// True if two RECTs overlap
fn rect_intersects(a: RECT, b: RECT) -> bool {
    a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top
}

pub unsafe fn create_window(hinstance: HINSTANCE) -> Result<HWND> {
    let cls = HSTRING::from(WINDOW_CLASS);
    let hicon = LoadIconW(hinstance, PCWSTR(1 as *const u16)).unwrap_or_default();
    let wc = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        hInstance: hinstance,
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        hbrBackground: CreateSolidBrush(COLORREF(0x002D2D2D)),
        hIcon: hicon,
        lpszClassName: PCWSTR(cls.as_ptr()),
        ..Default::default()
    };
    if RegisterClassW(&wc) == 0 { return Err(Error::from_win32()); }

    let n = with_window_state(|s| s.accounts.len().max(1)).unwrap_or(1) as i32;
    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
    let mut wr = RECT { left: 0, top: 0, right: 340, bottom: (10 + n * 40 + 32).min(768) };
    let _ = AdjustWindowRect(&mut wr, style, false);

    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0), PCWSTR(cls.as_ptr()), w!("TOTP Authenticator"),
        style | WS_VISIBLE, CW_USEDEFAULT, CW_USEDEFAULT,
        wr.right - wr.left, wr.bottom - wr.top,
        None, None, hinstance, None,
    );
    if hwnd.0 == 0 { return Err(Error::from_win32()); }
    Ok(hwnd)
}

/// Window procedure
unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let hmod = HINSTANCE(GetModuleHandleW(None).unwrap().0);
            with_window_state(|s| {
                for i in 0..s.accounts.len() {
                    CreateWindowExW(
                        WINDOW_EX_STYLE(0), w!("BUTTON"), w!("Copy"),
                        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_OWNERDRAW as u32),
                        265, 10 + (i as i32 * 40) - 5, 56, 25,
                        hwnd, HMENU((ID_COPY_BUTTON_BASE + i as i32) as isize), hmod, None,
                    );
                }
            });
            DARK_BRUSH.with(|b|        *b.borrow_mut() = CreateSolidBrush(COLORREF(0x002D2D2D)));
            BTN_NORMAL_BRUSH.with(|b|  *b.borrow_mut() = CreateSolidBrush(COLORREF(0x004A4A4A)));
            BTN_PRESSED_BRUSH.with(|b| *b.borrow_mut() = CreateSolidBrush(COLORREF(0x00666666)));
            BTN_BORDER_BRUSH.with(|b|  *b.borrow_mut() = CreateSolidBrush(COLORREF(0x00888888)));
            if let Ok(hicon) = LoadIconW(hmod, PCWSTR(1 as *const u16)) {
                let _ = SendMessageW(hwnd, WM_SETICON, WPARAM(1), LPARAM(hicon.0));
                let _ = SendMessageW(hwnd, WM_SETICON, WPARAM(0), LPARAM(hicon.0));
            }
            SetTimer(hwnd, TIMER_UPDATE, 1000, None);
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_UPDATE {
                let step = current_step();
                let n = with_window_state(|s| s.accounts.len()).unwrap_or(0) as i32;
                let timer_y = 10 + n * 40 + 10;
                // On 30s rollover: invalidate code column so new codes paint
                let prev = CODE_CACHE.with(|c| c.borrow().0);
                if step != prev {
                    let codes = RECT { left: 0, top: 0, right: 340, bottom: timer_y };
                    let _ = InvalidateRect(hwnd, Some(&codes), false);
                }
                // Every second: only the ~32px countdown strip
                let countdown = RECT { left: 10, top: timer_y, right: 340, bottom: timer_y + 32 };
                let _ = InvalidateRect(hwnd, Some(&countdown), false);
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            paint_window(hdc, ps.rcPaint);
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_SIZE => {
            // Stop timer when minimized, restart when restored to save CPU
            let size_type = wparam.0 as u32;
            if size_type == SIZE_MINIMIZED {
                let _ = KillTimer(hwnd, TIMER_UPDATE);
            } else if size_type == SIZE_RESTORED || size_type == SIZE_MAXIMIZED {
                let _ = SetTimer(hwnd, TIMER_UPDATE, 1000, None);
            }
            LRESULT(0)
        }
        WM_DRAWITEM => {
            if lparam.0 != 0 {
                let dis = &*(lparam.0 as *const DRAWITEMSTRUCT);
                let pressed = (dis.itemState.0 & 0x0001) != 0; // ODS_SELECTED
                let fill = if pressed {
                    BTN_PRESSED_BRUSH.with(|b| *b.borrow())
                } else {
                    BTN_NORMAL_BRUSH.with(|b| *b.borrow())
                };
                FillRect(dis.hDC, &dis.rcItem, fill);
                BTN_BORDER_BRUSH.with(|b| FrameRect(dis.hDC, &dis.rcItem, *b.borrow()));
                SetTextColor(dis.hDC, COLORREF(0x00E0E0E0));
                SetBkMode(dis.hDC, TRANSPARENT);
                let text = HSTRING::from("Copy");
                let mut rect = dis.rcItem;
                let mut slice = text.as_wide().to_vec();
                DrawTextW(dis.hDC, &mut slice, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
            }
            LRESULT(1)
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as i32 - ID_COPY_BUTTON_BASE;
            if id >= 0 {
                if let Some(Some(code)) = with_window_state(|s| s.accounts.get(id as usize).and_then(|a| a.current_code().ok())) {
                    if let Err(e) = crate::clipboard::copy_to_clipboard(&code) {
                        let msg = HSTRING::from(format!("Copy failed: {e}"));
                        MessageBoxW(hwnd, PCWSTR(msg.as_ptr()), w!("Error"), MB_OK | MB_ICONERROR);
                    }
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            let _ = KillTimer(hwnd, TIMER_UPDATE);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}


/// Paint window content — only redraws regions intersecting rcpaint
unsafe fn paint_window(hdc: HDC, rcpaint: RECT) {
    SetBkColor(hdc, COLORREF(0x002D2D2D));
    SetTextColor(hdc, COLORREF(0x00E0E0E0));
    let step = current_step();
    with_window_state(|state| {
        if state.accounts.is_empty() {
            let text = HSTRING::from("No accounts configured. Please edit secrets.txt");
            let mut rect = RECT { left: 10, top: 10, right: 330, bottom: 50 };
            let mut slice = text.as_wide().to_vec();
            DrawTextW(hdc, &mut slice, &mut rect, DT_LEFT | DT_VCENTER);
            return;
        }
        let n = state.accounts.len();
        // Refresh code cache only on 30s rollover — avoids HMAC-SHA1 every second
        CODE_CACHE.with(|cache| {
            let mut c = cache.borrow_mut();
            if c.0 != step {
                c.0 = step;
                c.1 = state.accounts.iter()
                    .map(|a| a.current_code().unwrap_or_default())
                    .collect();
            }
        });
        // Account rows — skip any row fully outside the dirty rect
        for i in 0..n {
            let y = 10 + (i as i32 * 40);
            if !rect_intersects(RECT { left: 0, top: y, right: 340, bottom: y + 40 }, rcpaint) {
                continue;
            }
            let name = HSTRING::from(&state.accounts[i].name);
            let mut name_rect = RECT { left: 10, top: y, right: 200, bottom: y + 30 };
            let mut name_slice = name.as_wide().to_vec();
            DrawTextW(hdc, &mut name_slice, &mut name_rect, DT_LEFT | DT_VCENTER);
            CODE_CACHE.with(|cache| {
                let c = cache.borrow();
                if let Some(code) = c.1.get(i) {
                    let code_text = HSTRING::from(code.as_str());
                    let mut code_rect = RECT { left: 210, top: y, right: 265, bottom: y + 30 };
                    let mut code_slice = code_text.as_wide().to_vec();
                    DrawTextW(hdc, &mut code_slice, &mut code_rect, DT_LEFT | DT_VCENTER);
                }
            });
        }
        // Countdown row — small dirty rect every tick, rarely skipped
        let timer_y = 10 + (n as i32 * 40) + 10;
        let timer_rect = RECT { left: 10, top: timer_y, right: 340, bottom: timer_y + 30 };
        if rect_intersects(timer_rect, rcpaint) {
            let remaining = state.accounts.first()
                .map(|a| a.time_remaining().max(1))
                .unwrap_or(30);
            let timer_text = HSTRING::from(format!("Next code in: {:2}s   ", remaining));
            let mut t_rect = timer_rect;
            let mut slice = timer_text.as_wide().to_vec();
            DrawTextW(hdc, &mut slice, &mut t_rect, DT_LEFT | DT_VCENTER);
        }
        // Error line (static — only redrawn when code area is dirty)
        if state.invalid_line_count > 0 {
            let y = 10 + (n as i32 * 40) + 50;
            let err_rect = RECT { left: 10, top: y, right: 330, bottom: y + 30 };
            if rect_intersects(err_rect, rcpaint) {
                SetTextColor(hdc, COLORREF(0x000000FF));
                let msg = HSTRING::from(format!("{} invalid lines in secrets.txt", state.invalid_line_count));
                let mut e_rect = err_rect;
                let mut e_slice = msg.as_wide().to_vec();
                DrawTextW(hdc, &mut e_slice, &mut e_rect, DT_LEFT | DT_VCENTER);
            }
        }
    });
}



