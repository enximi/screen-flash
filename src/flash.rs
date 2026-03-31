use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, HGDIOBJ, PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetSystemMetrics, HMENU,
    HWND_TOPMOST, LWA_ALPHA, MSG, PM_REMOVE, PeekMessageW, PostQuitMessage, RegisterClassW,
    SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
    SW_SHOWNOACTIVATE, SWP_SHOWWINDOW, SetLayeredWindowAttributes, SetWindowPos, ShowWindow,
    TranslateMessage, UnregisterClassW, WM_DESTROY, WM_PAINT, WM_QUIT, WNDCLASSW, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};
use windows::core::{Error, PCWSTR, Result, w};

use crate::color::FlashColor;

const FLASH_HOLD_MS: u64 = 90;
const FLASH_FADE_MS: u64 = 240;
const FLASH_DURATION_MS: u64 = FLASH_HOLD_MS + FLASH_FADE_MS;
const FLASH_MAX_ALPHA: u8 = 160;
const FLASH_STEP_MS: u64 = 16;

static FLASH_COLOR: AtomicU32 = AtomicU32::new(colorref_from_flash_color(FlashColor {
    red: 0,
    green: 0,
    blue: 0,
}));
pub(crate) fn flash_screen(color: FlashColor) -> Result<()> {
    FLASH_COLOR.store(colorref_from_flash_color(color), Ordering::Relaxed);

    let class_name = w!("ScreenFlashWindow");
    let instance = get_module_handle()?;

    let window_class = WNDCLASSW {
        lpfnWndProc: Some(flash_window_proc),
        hInstance: instance,
        lpszClassName: class_name,
        ..Default::default()
    };

    let class_atom = unsafe { RegisterClassW(&window_class) };
    if class_atom == 0 {
        return Err(Error::from_thread());
    }

    let flash_result = create_flash_window(instance, class_name).and_then(run_flash_animation);
    let unregister_result = unsafe { UnregisterClassW(class_name, Some(instance)) };

    if flash_result.is_ok() {
        unregister_result?;
    }

    flash_result
}

fn create_flash_window(instance: HINSTANCE, class_name: PCWSTR) -> Result<HWND> {
    let x = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let y = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };

    let window = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
            class_name,
            PCWSTR::null(),
            WS_POPUP,
            x,
            y,
            width,
            height,
            None,
            None::<HMENU>,
            Some(instance),
            None,
        )?
    };

    unsafe {
        SetLayeredWindowAttributes(window, COLORREF(0), FLASH_MAX_ALPHA, LWA_ALPHA)?;
        let _ = ShowWindow(window, SW_SHOWNOACTIVATE);
        SetWindowPos(
            window,
            Some(HWND_TOPMOST),
            x,
            y,
            width,
            height,
            SWP_SHOWWINDOW,
        )?;
    }

    Ok(window)
}

fn run_flash_animation(window: HWND) -> Result<()> {
    let started_at = Instant::now();

    loop {
        if !pump_window_messages() {
            break;
        }

        let elapsed_ms = started_at.elapsed().as_millis() as u64;
        if elapsed_ms >= FLASH_DURATION_MS {
            break;
        }

        let alpha = alpha_for_elapsed_ms(elapsed_ms);
        unsafe {
            SetLayeredWindowAttributes(window, COLORREF(0), alpha, LWA_ALPHA)?;
        }

        thread::sleep(Duration::from_millis(FLASH_STEP_MS));
    }

    unsafe {
        SetLayeredWindowAttributes(window, COLORREF(0), 0, LWA_ALPHA)?;
        DestroyWindow(window)?;
    }
    let _ = pump_window_messages();

    Ok(())
}

fn alpha_for_elapsed_ms(elapsed_ms: u64) -> u8 {
    if elapsed_ms <= FLASH_HOLD_MS {
        return FLASH_MAX_ALPHA;
    }

    let fade_elapsed_ms = elapsed_ms.saturating_sub(FLASH_HOLD_MS);
    let remaining_ms = FLASH_FADE_MS.saturating_sub(fade_elapsed_ms);
    let scaled = u64::from(FLASH_MAX_ALPHA) * remaining_ms / FLASH_FADE_MS;
    scaled as u8
}

fn pump_window_messages() -> bool {
    let mut message = MSG::default();

    unsafe {
        while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
            if message.message == WM_QUIT {
                return false;
            }

            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    true
}

unsafe extern "system" fn flash_window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            let dc = unsafe { BeginPaint(window, &mut paint) };
            let brush = unsafe { CreateSolidBrush(current_flash_color()) };
            unsafe {
                FillRect(dc, &paint.rcPaint, brush);
                let _ = DeleteObject(HGDIOBJ(brush.0));
                let _ = EndPaint(window, &paint);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn get_module_handle() -> Result<HINSTANCE> {
    let module = unsafe { GetModuleHandleW(PCWSTR::null())? };
    Ok(module.into())
}

fn current_flash_color() -> COLORREF {
    COLORREF(FLASH_COLOR.load(Ordering::Relaxed))
}

const fn colorref_from_flash_color(color: FlashColor) -> u32 {
    (color.red as u32) | ((color.green as u32) << 8) | ((color.blue as u32) << 16)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_decreases_over_time() {
        assert_eq!(alpha_for_elapsed_ms(0), FLASH_MAX_ALPHA);
        assert_eq!(alpha_for_elapsed_ms(FLASH_HOLD_MS / 2), FLASH_MAX_ALPHA);
        assert!(alpha_for_elapsed_ms(FLASH_HOLD_MS + (FLASH_FADE_MS / 2)) < FLASH_MAX_ALPHA);
        assert_eq!(alpha_for_elapsed_ms(FLASH_DURATION_MS), 0);
    }

    #[test]
    fn converts_rgb_to_colorref_layout() {
        assert_eq!(
            colorref_from_flash_color(FlashColor {
                red: 0x12,
                green: 0x34,
                blue: 0x56,
            }),
            0x0056_3412
        );
    }
}
