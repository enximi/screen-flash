use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, HGDIOBJ, InvalidateRect,
    PAINTSTRUCT,
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
use crate::effect::{FlashEffect, FlashSample};

static FLASH_COLOR: AtomicU32 = AtomicU32::new(colorref_from_flash_color(FlashColor {
    red: 0,
    green: 0,
    blue: 0,
}));

/// 使用给定效果在全屏范围内执行一次闪烁。
pub fn flash_screen<E>(effect: E) -> Result<()>
where
    E: FlashEffect,
{
    let initial_sample = effect.sample(0);
    store_flash_color(initial_sample.color);

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

    let flash_result = create_flash_window(instance, class_name, alpha_to_u8(initial_sample.alpha))
        .and_then(|window| run_flash_effect(window, effect));
    let unregister_result = unsafe { UnregisterClassW(class_name, Some(instance)) };

    match (flash_result, unregister_result) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
    }
}

fn create_flash_window(instance: HINSTANCE, class_name: PCWSTR, initial_alpha: u8) -> Result<HWND> {
    let bounds = virtual_screen_bounds();

    let window = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
            class_name,
            PCWSTR::null(),
            WS_POPUP,
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            None,
            None::<HMENU>,
            Some(instance),
            None,
        )?
    };

    unsafe {
        SetLayeredWindowAttributes(window, COLORREF(0), initial_alpha, LWA_ALPHA)?;
        let _ = ShowWindow(window, SW_SHOWNOACTIVATE);
        SetWindowPos(
            window,
            Some(HWND_TOPMOST),
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            SWP_SHOWWINDOW,
        )?;
    }

    Ok(window)
}

fn run_flash_effect<E>(window: HWND, effect: E) -> Result<()>
where
    E: FlashEffect,
{
    let started_at = Instant::now();

    loop {
        if !pump_window_messages() {
            break;
        }

        let elapsed_ms = started_at.elapsed().as_millis() as u64;
        let sample = effect.sample(elapsed_ms);
        apply_sample(window, sample)?;

        match sample.next_step_ms {
            Some(step_ms) => thread::sleep(Duration::from_millis(step_ms.max(1))),
            None => break,
        }
    }

    unsafe {
        SetLayeredWindowAttributes(window, COLORREF(0), 0, LWA_ALPHA)?;
        DestroyWindow(window)?;
    }
    let _ = pump_window_messages();

    Ok(())
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

fn apply_sample(window: HWND, sample: FlashSample) -> Result<()> {
    store_flash_color(sample.color);

    unsafe {
        SetLayeredWindowAttributes(window, COLORREF(0), alpha_to_u8(sample.alpha), LWA_ALPHA)?;
        let _ = InvalidateRect(Some(window), None, false);
    }

    let _ = pump_window_messages();

    Ok(())
}

fn alpha_to_u8(alpha: f32) -> u8 {
    (alpha.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn store_flash_color(color: FlashColor) {
    FLASH_COLOR.store(colorref_from_flash_color(color), Ordering::Relaxed);
}

fn virtual_screen_bounds() -> ScreenBounds {
    ScreenBounds {
        x: unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) },
        y: unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) },
        width: unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) },
        height: unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) },
    }
}

const fn colorref_from_flash_color(color: FlashColor) -> u32 {
    (color.red as u32) | ((color.green as u32) << 8) | ((color.blue as u32) << 16)
}

struct ScreenBounds {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn clamps_alpha_before_converting_to_u8() {
        assert_eq!(alpha_to_u8(-1.0), 0);
        assert_eq!(alpha_to_u8(0.5), 128);
        assert_eq!(alpha_to_u8(2.0), 255);
    }
}
