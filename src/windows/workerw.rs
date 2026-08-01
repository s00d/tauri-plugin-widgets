//! Windows wallpaper WorkerW parenting — separate from Widgets Board / Adaptive Cards.
//!
//! Enable with feature `workerw`. Attach a frameless Tauri webview HWND behind icons
//! via the classic Progman / WorkerW technique. Not used by `IWidgetProvider`.

#![cfg(all(target_os = "windows", feature = "workerw"))]

use std::ffi::c_void;

type BOOL = i32;
type HWND = *mut c_void;

#[link(name = "user32")]
unsafe extern "system" {
    fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> HWND;
    fn FindWindowExW(
        hWndParent: HWND,
        hWndChildAfter: HWND,
        lpszClass: *const u16,
        lpszWindow: *const u16,
    ) -> HWND;
    fn SendMessageTimeoutW(
        hWnd: HWND,
        Msg: u32,
        wParam: usize,
        lParam: isize,
        fuFlags: u32,
        uTimeout: u32,
        lpdwResult: *mut usize,
    ) -> usize;
    fn SetParent(hWndChild: HWND, hWndNewParent: HWND) -> HWND;
    fn EnumWindows(
        lpEnumFunc: unsafe extern "system" fn(HWND, isize) -> BOOL,
        lParam: isize,
    ) -> BOOL;
}

const SMTO_NORMAL: u32 = 0;
const WM_SPAWN_WORKER: u32 = 0x052C;

struct EnumState {
    workerw: HWND,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: isize) -> BOOL {
    let state = &mut *(lparam as *mut EnumState);
    let shelldll = wide("SHELLDLL_DefView");
    let def = FindWindowExW(hwnd, std::ptr::null_mut(), shelldll.as_ptr(), std::ptr::null());
    if !def.is_null() {
        let worker = wide("WorkerW");
        state.workerw = FindWindowExW(std::ptr::null_mut(), hwnd, worker.as_ptr(), std::ptr::null());
    }
    1
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Find the WorkerW HWND used as the desktop wallpaper layer (may be null).
pub fn find_workerw() -> Option<isize> {
    unsafe {
        let progman_class = wide("Progman");
        let progman = FindWindowW(progman_class.as_ptr(), std::ptr::null());
        if progman.is_null() {
            return None;
        }
        let mut result: usize = 0;
        SendMessageTimeoutW(
            progman,
            WM_SPAWN_WORKER,
            0xD,
            0,
            SMTO_NORMAL,
            1000,
            &mut result,
        );
        let mut state = EnumState {
            workerw: std::ptr::null_mut(),
        };
        EnumWindows(enum_windows_proc, &mut state as *mut _ as isize);
        if state.workerw.is_null() {
            None
        } else {
            Some(state.workerw as isize)
        }
    }
}

/// Reparent `child` HWND under WorkerW. Returns false if WorkerW missing.
pub fn attach_to_workerw(child_hwnd: isize) -> bool {
    let Some(worker) = find_workerw() else {
        return false;
    };
    unsafe {
        SetParent(child_hwnd as HWND, worker as HWND);
        true
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn api_compiles() {
        // Runtime requires interactive desktop; compile-only here.
        assert!(true);
    }
}
