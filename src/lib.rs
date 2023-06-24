use std::default::Default;
use core::ptr::null_mut;
use std::sync::Mutex;

use winapi::ctypes::c_int;
use winapi::shared::minwindef::BOOL;

use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleW;

use winapi::um::winuser::{
    WNDCLASSW,
    WNDPROC,
    MSG,
    IDC_ARROW,
    SMTO_NORMAL,

    PM_REMOVE,
    WS_POPUP,
    WS_VISIBLE,
    SM_CXSCREEN,
    SM_CYSCREEN,
    SWP_NOZORDER,
    SWP_NOOWNERZORDER,
};

use winapi::um::winuser::{
    RegisterClassW,
    CreateWindowExW,
    ShowWindow,
    EnumWindows,
    FindWindowW,
    FindWindowExW,
    SendMessageTimeoutW,

    GetSystemMetrics,
    SetWindowPos,
    SetParent,

    LoadCursorW,
};

use winapi::um::winuser::{
    PeekMessageW,
    TranslateMessage,
    DispatchMessageW,
    SystemParametersInfoW,
};

use winapi::shared::minwindef::{
    HINSTANCE,
    LPARAM,
};

use winapi::um::winuser::SW_SHOW;
use winapi::shared::windef::HWND;

/// Used in FindWindowExW(). Is the name of the window class that is the parent of the desktop window:
///
/// --- Window ... SHELLDLL_DefView
///
/// ------ Window ... WorkerW
///
/// --------- OurWindow ... OurWindowClass
///
/// See more: <https://learn.microsoft.com/en-us/archive/msdn-magazine/2004/march/c-q-a-list-view-mode-setforegroundwindow-and-class-protection>
/// <https://www.codeproject.com/Articles/856020/Draw-Behind-Desktop-Icons-in-Windows-plus>
pub const SHELLDLL_DEF_VIEW_STR : &str = "SHELLDLL_DefView";

/// Used in FindWindowExW(). Any application that needs to listen to window messages call this Api to create a worker window.
/// Is the name of the window class we are looking for to put our window into as a child:
///
/// --- Window ... SHELLDLL_DefView
///
/// ------ Window ... WorkerW
///
/// --------- OurWindow ... OurWindowClass
///
/// See more: <https://learn.microsoft.com/en-us/archive/msdn-magazine/2004/march/c-q-a-list-view-mode-setforegroundwindow-and-class-protection>
/// <https://www.codeproject.com/Articles/856020/Draw-Behind-Desktop-Icons-in-Windows-plus>
pub const WORKER_W_STR : &str = "WorkerW";

pub mod drawing;

/// Handle to desktop window app. Any application that needs to listen to window messages call this Api to create a worker window.
static mut WORKER_W : Mutex::<HWND> = Mutex::new(null_mut());

/// Create WNDCLASSW and handle to it with custom name and WNDPROC.
///
/// <i>window_procedure</i> - A callback function, which you define in your application, that processes messages sent to a window.
///
/// Read more:
///
/// WNDCLASSW - <https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassw>
///
/// WNDPROC - <https://learn.microsoft.com/en-us/windows/win32/api/winuser/nc-winuser-wndproc>
///
/// Procedure example:
/// ```
/// pub unsafe extern "system" fn window_procedure(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM,) -> LRESULT {
///    match msg {
///        WM_NCCREATE => {
///            println!("NC Create");
///            let createstruct: *mut CREATESTRUCTW = l_param as *mut _;
///            if createstruct.is_null() {
///                return 0;
///            }
///            let boxed_i32_ptr = (*createstruct).lpCreateParams;
///            SetWindowLongPtrW(hwnd, GWLP_USERDATA, boxed_i32_ptr as LONG_PTR);
///            return 1;
///        }
///        WM_CREATE => println!("WM Create"),
///        WM_CLOSE => drop(DestroyWindow(hwnd)),
///        WM_DESTROY => {
///            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut i32;
///            drop(Box::from_raw(ptr));
///            println!("Cleaned up the box.");
///            PostQuitMessage(0);
///        }
///        WM_ERASEBKGND => return 1,
///        WM_PAINT => your_paint_func(hwnd),
///        _ => return DefWindowProcW(hwnd, msg, w_param, l_param),
///    }
///
///    0
///  }
/// ```
pub fn create_window_class(name: &Vec<u16>, window_procedure: WNDPROC) -> (WNDCLASSW, HINSTANCE) {
    let h_instance = unsafe { GetModuleHandleW(core::ptr::null()) };

    let mut wc = WNDCLASSW::default();
    wc.lpfnWndProc = window_procedure;
    wc.hInstance = h_instance;
    wc.lpszClassName = name.as_ptr();
    wc.hCursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
    (wc, h_instance)
}

/// Create window handle for window class (WNDCLASSW) with <i>window_name</i>
///
/// <i>wc</i> and <i>h_instance</i> - can be results o the [`create_window_class()`] func
pub fn create_window_handle(wc: &WNDCLASSW, wc_name: &Vec<u16>, window_name: &Vec<u16>, h_instance: HINSTANCE, ) -> HWND {
    let atom = unsafe { RegisterClassW(wc) };
    if atom == 0 {
        let last_error = unsafe { GetLastError() };
        panic!("Could not register the window class, error code: {}", last_error);
    }

    let lparam: *mut i32 = Box::leak(Box::new(5_i32));
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            wc_name.as_ptr(),
            window_name.as_ptr(),
            WS_POPUP | WS_VISIBLE,
            0,
            0,
            0,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            h_instance,
            lparam.cast(),
        )
    };
    if hwnd.is_null() {
        panic!("Failed to create a window.");
    }

    hwnd
}

/// Create window using window <i>handle</i>.
///
/// <i>handle</i> - can be result of [`create_window_handle()`] func
pub fn create_window(handle: HWND) {
    let _previously_visible = unsafe { ShowWindow(handle, SW_SHOW) };
}

/// Find `Progman` and get handle. Progman requires for [`try_spawn_worker_w()`] func
pub fn get_progman_handle() -> HWND {
    let h_progman = unsafe { FindWindowW(wide_null("Progman").as_ptr(), null_mut()) };
    h_progman
}

/// Message to `Progman` to spawn a `WorkerW`
///
/// Send 0x052C to Progman. This message directs Progman to spawn a
/// WorkerW behind the desktop icons. If it is already there, nothing
/// happens.
pub fn try_spawn_worker_w(progman_handle: HWND) -> Result<(), &'static str> {
    // Requare all for support all windows versions!
    let send_message_results = unsafe { [
        SendMessageTimeoutW(progman_handle, 0x052C, 0, 0, SMTO_NORMAL, 1000, null_mut()),
        SendMessageTimeoutW(progman_handle, 0x052C, 0x0d, 0, SMTO_NORMAL, 1000, null_mut()),
        SendMessageTimeoutW(progman_handle, 0x052C, 0x0d, 1, SMTO_NORMAL, 1000, null_mut())
    ] };

    if send_message_results.iter().all(|r| *r == 0) {
        return Err("`Progman` failed to spawn WorkerW!");
    }

    Ok(())
}

/// Find the newly created `WorkerW`
pub fn find_worker_w() -> HWND {
    unsafe {
        EnumWindows(Some(enum_windows_proc), 0);
        return WORKER_W.lock().unwrap().clone();
    };
}

/// Sets worker_w_handle as parent to handle and set window size to [`winapi::um::winuser::SM_CXSCREEN`] x [`winapi::um::winuser::SM_CYSCREEN`]
///
/// Used flags:
///
/// <b>SWP_NOOWNERZORDER</b> - Does not change the owner window's position in the Z order.
///
/// <b>SWP_NOZORDER</b> - Retains the current Z order (ignores the hWndInsertAfter parameter).
pub fn pull_window_to_desktop(handle: HWND, worker_w_handle: HWND) {
    unsafe { SetParent(handle, worker_w_handle) };
    unsafe {
        SetWindowPos(
            handle,
            null_mut(),
            0,
            0,
            GetSystemMetrics(SM_CXSCREEN) as c_int,
            GetSystemMetrics(SM_CYSCREEN) as c_int,
            SWP_NOOWNERZORDER | SWP_NOZORDER
        )
    };

    unsafe { SystemParametersInfoW(20, 0, null_mut(), 0x1) };
}

/// It receives top-level window handles and find windows with class [`SHELLDLL_DEF_VIEW_STR`] + child with [`WORKER_W_STR`] class
///
/// Read more: <https://learn.microsoft.com/ru-ru/previous-versions/windows/desktop/legacy/ms633498(v=vs.85)>
pub unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _l_param: LPARAM) -> BOOL {
    let shelldll_def_view_name = wide_null(SHELLDLL_DEF_VIEW_STR);
    let cur_hwnd = unsafe { FindWindowExW(hwnd, null_mut(), shelldll_def_view_name.as_ptr(), null_mut()) };

    if !cur_hwnd.is_null()
    {
        println!("{} window found!", SHELLDLL_DEF_VIEW_STR);
        let worker_w_name = wide_null(WORKER_W_STR);
        // Gets the WorkerW Window after the current one.
        let mut worker = WORKER_W.lock().unwrap();
        unsafe { *worker = FindWindowExW(null_mut(), hwnd, worker_w_name.as_ptr(), null_mut()) };
        if !worker.is_null() {
            println!("{} window found!", WORKER_W_STR);
        }
    }

    return 1;
}

/// A simple function to handle window messages.
/// You can use it, or define your own. It use PeekMessageW() (<https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-peekmessagew>)
///
/// Returns TRUE if the message was received and processed
///
/// Example:
/// ```
/// let msg = MSG::default();
/// loop {
///     if handle_window_messages(msg) {
///         println!("Message received and processed!");
///     }
///     else {
///         std::thread::sleep(std::time::Duration::from_micros(100));
///     }
/// }
/// ```
pub fn handle_window_messages(mut msg: MSG) -> bool {
    let message_return = unsafe { PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) };
    if message_return == 0 {
        return false;
    } else if message_return == -1 {
        let last_error = unsafe { GetLastError() };
        panic!("Error with `GetMessageW`, error code: {}", last_error);
    } else {
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    true
}

/// Combines low-lewel methods for simplify create window at desktop!
///
/// <i>name</i> - name of new window
///
/// <i>window_procedure</i> - A callback function, which you define in your application, that processes messages sent to a window.
///
/// Read more about WNDPROC - <https://learn.microsoft.com/en-us/windows/win32/api/winuser/nc-winuser-wndproc>
///
/// Procedure example:
/// ```
/// pub unsafe extern "system" fn window_procedure(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM,) -> LRESULT {
///    match msg {
///        WM_NCCREATE => {
///            println!("NC Create");
///            let createstruct: *mut CREATESTRUCTW = l_param as *mut _;
///            if createstruct.is_null() {
///                return 0;
///            }
///            let boxed_i32_ptr = (*createstruct).lpCreateParams;
///            SetWindowLongPtrW(hwnd, GWLP_USERDATA, boxed_i32_ptr as LONG_PTR);
///            return 1;
///        }
///        WM_CREATE => println!("WM Create"),
///        WM_CLOSE => drop(DestroyWindow(hwnd)),
///        WM_DESTROY => {
///            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut i32;
///            drop(Box::from_raw(ptr));
///            println!("Cleaned up the box.");
///            PostQuitMessage(0);
///        }
///        WM_ERASEBKGND => return 1,
///        WM_PAINT => your_paint_func(hwnd),
///        _ => return DefWindowProcW(hwnd, msg, w_param, l_param),
///    }
///
///    0
///  }
/// ```
pub fn create_desktop_window_fast(name: &str, window_procedure: WNDPROC) -> HWND {
    let class_name = wide_null(format!("{} Class", name).as_str());
    let window_name = wide_null(name);
    let (window_class, h_instance) = create_window_class(&class_name, window_procedure);
    let window_handle = create_window_handle(&window_class, &class_name, &window_name, h_instance);
    create_window(window_handle);

    let progman_h = get_progman_handle();
    if try_spawn_worker_w(progman_h).is_err() {
        panic!("`Progman` failed to spawn WorkerW!");
    };

    let worker_w_handle = find_worker_w();
    pull_window_to_desktop(window_handle, worker_w_handle);

    window_handle
}

/// Convert string to windows friedly format.
///
/// !!! With this strings use W-functions !!!
///
/// For example: [`winapi::um::libloaderapi::GetModuleHandleW()`] or [`winapi::um::winuser::RegisterClassW`]
pub fn wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}