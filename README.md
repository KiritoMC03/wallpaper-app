# wallpaper-app

This library is designed to make it easy to create your own desktop.
You can use it to create an application window and place it in the desktop so that your window is below the icons.

## For now, it supports only Windows!

## Examples
An example of using the library in a project: https://github.com/KiritoMC03/live-wallpapers

```toml
[dependencies]
wallpaper-app = "0.1.1"
winapi = { version = "0.3.9", features = ["winuser", "processthreadsapi", "libloaderapi", "errhandlingapi", "impl-default"] }
```

### Code example:

```rust
use winapi::shared::windef::HWND;
use winapi::shared::basetsd::LONG_PTR;

use winapi::shared::minwindef::{
    LPARAM,
    LRESULT,
    UINT,
    WPARAM,
};

use winapi::um::winuser::{
    CREATESTRUCTW,

    SetProcessDPIAware,
    DefWindowProcW,
    PostQuitMessage,
    DestroyWindow,
    GetWindowLongPtrW,
    SetWindowLongPtrW,

    GWLP_USERDATA,
    WM_CLOSE,
    WM_CREATE,
    WM_DESTROY,
    WM_NCCREATE,
    WM_PAINT,
    WM_ERASEBKGND,
};

use wallpaper_app::create_desktop_window_fast;

fn main() {
    // Sets the process-default DPI awareness to system-DPI awareness.
    // Allows you to ignore interface scaling in Windows.
    unsafe { SetProcessDPIAware(); }
    let window_handle = create_desktop_window_fast("Live", Some(window_procedure));
    // Some code...
    build_app();
    loop_graphics(window_handle);
}

// A callback function, which you define in your application, that processes messages sent to a window.
pub unsafe extern "system" fn window_procedure(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM,) -> LRESULT {
    match msg {
        WM_NCCREATE => {
            println!("NC Create");
            // About this code and GWLP_USERDATA you can read here:
            // https://learn.microsoft.com/en-us/windows/win32/learnwin32/managing-application-state-
            let createstruct: *mut CREATESTRUCTW = l_param as *mut _;
            if createstruct.is_null() {
                return 0;
            }
            let boxed_i32_ptr = (*createstruct).lpCreateParams;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, boxed_i32_ptr as LONG_PTR);
            return 1;
        }
        WM_CREATE => println!("WM Create"),
        WM_CLOSE => drop(DestroyWindow(hwnd)),
        WM_DESTROY => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut i32;
            drop(Box::from_raw(ptr));
            println!("Cleaned up the box.");
            PostQuitMessage(0);
        }
        WM_ERASEBKGND => return 1,
        WM_PAINT => paint_frame(hwnd, get_app_data()), // paint_frame() - your function
        _ => return DefWindowProcW(hwnd, msg, w_param, l_param),
    }

    0
}
```