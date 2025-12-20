#![windows_subsystem = "windows"]

mod audio;
mod window;

use std::{io::Cursor, mem, ptr, slice, thread, time::Duration};

use kira::{
    clock::ClockSpeed,
    manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
    sound::streaming::{StreamingSoundData, StreamingSoundSettings},
};

use windows::{
    core::s,
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM},
        Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, SelectObject,
            AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION,
            DIB_RGB_COLORS,
        },
        System::{
            Com::{CoInitialize, CoUninitialize},
            LibraryLoader::GetModuleHandleA,
            Threading::{GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_HIGHEST},
        },
        UI::{
            HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
            WindowsAndMessaging::{
                CreateWindowExA, DefWindowProcA, DispatchMessageA, GetSystemMetrics, PeekMessageA,
                PostQuitMessage, RegisterClassA, TranslateMessage, UpdateLayeredWindow, MSG,
                PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, ULW_ALPHA, WM_CLOSE, WM_DESTROY,
                WM_ERASEBKGND, WM_QUIT, WNDCLASSA, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
                WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE,
            },
        },
    },
};

const BIN_DATA: &[u8] = include_bytes!(env!("BIN_PATH"));
const AUDIO_DATA: &[u8] = include_bytes!(env!("AUDIO_PATH"));

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_CLOSE => {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        _ => DefWindowProcA(hwnd, msg, w, l),
    }
}

fn main() {
    unsafe {
        let _ = CoInitialize(None);
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let _ = windows::Win32::Media::timeBeginPeriod(1);
        let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);

        let (fps_bytes, frames_bytes) = BIN_DATA.split_at(2);
        let video_fps = u16::from_le_bytes(fps_bytes.try_into().unwrap()) as f64;
        let frames: &[window::RawWinCoords] = slice::from_raw_parts(
            frames_bytes.as_ptr() as *const _,
            frames_bytes.len() / mem::size_of::<window::RawWinCoords>(),
        );
        let mut frames_iter = frames.iter();

        let instance: HINSTANCE = GetModuleHandleA(None).unwrap().into();
        let class_name = s!("PixelShell");
        let wc = WNDCLASSA {
            lpfnWndProc: Some(wnd_proc),
            hInstance: instance,
            lpszClassName: class_name,
            ..Default::default()
        };
        RegisterClassA(&wc);

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);

        let overlay_hwnd = CreateWindowExA(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name,
            s!(""),
            WS_POPUP | WS_VISIBLE,
            0,
            0,
            screen_w,
            screen_h,
            None,
            None,
            instance,
            None,
        );

        let mut manager =
            AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
        let clock = manager
            .add_clock(ClockSpeed::TicksPerSecond(video_fps))
            .unwrap();
        let sound_data = StreamingSoundData::from_cursor(
            Cursor::new(AUDIO_DATA),
            StreamingSoundSettings::new().start_time(clock.time()),
        )
        .unwrap();
        let volume_ctl = audio::get_volume_control();
        manager.play(sound_data).unwrap();
        clock.start().unwrap();

        let rx = screen_w as f64 / window::BASE_WIDTH as f64;
        let ry = screen_h as f64 / window::BASE_HEIGHT as f64;

        let screen_dc = GetDC(HWND(0));
        let mem_dc = CreateCompatibleDC(screen_dc);

        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: screen_w,
                biHeight: -screen_h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bits_ptr: *mut std::ffi::c_void = ptr::null_mut();
        let hbitmap =
            CreateDIBSection(screen_dc, &bmi, DIB_RGB_COLORS, &mut bits_ptr, None, 0).unwrap();
        SelectObject(mem_dc, hbitmap);

        let buffer_size = (screen_w * screen_h) as usize;
        let pixel_buffer = slice::from_raw_parts_mut(bits_ptr as *mut u32, buffer_size);

        let window_size = SIZE {
            cx: screen_w,
            cy: screen_h,
        };
        let zero_point = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
            ..Default::default()
        };

        let mut next_tick = clock.time().ticks;

        'main_loop: loop {
            if let Some(ref v) = volume_ctl {
                let _ = v.SetMasterVolumeLevelScalar(0.2, ptr::null());
                let _ = v.SetMute(false, ptr::null());
            }

            let mut msg = MSG::default();
            while PeekMessageA(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    break 'main_loop;
                }
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }

            let current_tick = clock.time().ticks;

            if current_tick >= next_tick {
                while current_tick > next_tick {
                    for c in frames_iter.by_ref() {
                        if c.w == 0 && c.h == 0 {
                            break;
                        }
                    }
                    next_tick += 1;
                }

                pixel_buffer.fill(0x00000000);

                loop {
                    let c = match frames_iter.next() {
                        Some(coords) => coords,
                        None => break 'main_loop,
                    };
                    if c.w == 0 && c.h == 0 {
                        break;
                    }

                    let left = (c.x as f64 * rx).round() as usize;
                    let top = (c.y as f64 * ry).round() as usize;
                    let w_scaled = (c.w as f64 * rx).round() as usize;
                    let h_scaled = (c.h as f64 * ry).round() as usize;

                    let right = (left + w_scaled).min(screen_w as usize);
                    let bottom = (top + h_scaled).min(screen_h as usize);

                    if right > left && bottom > top {
                        for y in top..bottom {
                            let row_offset = y * (screen_w as usize);
                            pixel_buffer[row_offset + left..row_offset + right].fill(0xFFFFFFFF);
                        }
                    }
                }

                let _ = UpdateLayeredWindow(
                    overlay_hwnd,
                    screen_dc,
                    Some(&zero_point),
                    Some(&window_size),
                    mem_dc,
                    Some(&zero_point),
                    COLORREF(0),
                    Some(&blend),
                    ULW_ALPHA,
                );

                next_tick += 1;
            } else {
                thread::sleep(Duration::from_millis(1));
            }
        }

        DeleteObject(hbitmap);
        DeleteDC(mem_dc);
        CoUninitialize();
    }
}
