//! Windows wallpaper layer for `layer: "desktop"` widget windows — separate from Widgets
//! Board / Adaptive Cards.
//!
//! [`attach_to_desktop`] reparents a frameless Tauri webview HWND behind the desktop icons:
//! - classic (Windows 10, 11 before 24H2): `0x052C` to Progman spawns a top-level WorkerW
//!   behind the `SHELLDLL_DefView` host; the window becomes its child.
//! - Windows 11 24H2+ (and whenever `SHELLDLL_DefView` is still a Progman child after
//!   `0x052C`): no top-level WorkerW; DefView and the wallpaper WorkerW are children of
//!   Progman, so the window becomes a Progman child right below `SHELLDLL_DefView`
//!   (above the wallpaper WorkerW).
//!
//! Attached windows are re-attached when Explorer restarts (`TaskbarCreated` broadcast,
//! received by one hidden top-level window on a watcher thread).

#![cfg(target_os = "windows")]
// Win32 type names as in the SDK headers.
#![allow(clippy::upper_case_acronyms)]

use std::ffi::c_void;
use std::sync::{Mutex, Once};

type BOOL = i32;
type HWND = *mut c_void;

#[repr(C)]
#[derive(Default)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct MSG {
    hwnd: HWND,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt: POINT,
    private: u32,
}

type WndProc = unsafe extern "system" fn(HWND, u32, usize, isize) -> isize;

#[repr(C)]
struct WNDCLASSW {
    style: u32,
    lpfn_wnd_proc: WndProc,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: *mut c_void,
    h_icon: *mut c_void,
    h_cursor: *mut c_void,
    hbr_background: *mut c_void,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
}

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
    ) -> isize;
    fn SetParent(hWndChild: HWND, hWndNewParent: HWND) -> HWND;
    fn EnumWindows(
        lpEnumFunc: unsafe extern "system" fn(HWND, isize) -> BOOL,
        lParam: isize,
    ) -> BOOL;
    fn IsWindow(hWnd: HWND) -> BOOL;
    fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    fn ScreenToClient(hWnd: HWND, lpPoint: *mut POINT) -> BOOL;
    fn SetWindowPos(
        hWnd: HWND,
        hWndInsertAfter: HWND,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> BOOL;
    fn RegisterWindowMessageW(lpString: *const u16) -> u32;
    fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: *mut c_void,
        hInstance: *mut c_void,
        lpParam: *mut c_void,
    ) -> HWND;
    fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> BOOL;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLastError() -> u32;
    fn SetLastError(dwErrCode: u32);
    fn GetModuleHandleW(lpModuleName: *const u16) -> *mut c_void;
}

const SMTO_NORMAL: u32 = 0;
const WM_SPAWN_WORKER: u32 = 0x052C;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
const WS_POPUP: u32 = 0x8000_0000;

/// Windows attached with [`attach_to_desktop`] (HWNDs as `isize`), re-attached on
/// `TaskbarCreated`. Destroyed ones are dropped on the next pass (`IsWindow`).
static ATTACHED: Mutex<Vec<isize>> = Mutex::new(Vec::new());
static WATCHER: Once = Once::new();

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: isize) -> BOOL {
    let workerw = &mut *(lparam as *mut HWND);
    let shelldll = wide("SHELLDLL_DefView");
    let def = FindWindowExW(
        hwnd,
        std::ptr::null_mut(),
        shelldll.as_ptr(),
        std::ptr::null(),
    );
    if !def.is_null() {
        // The WorkerW spawned by 0x052C is the next top-level sibling of the DefView host.
        let worker = wide("WorkerW");
        *workerw = FindWindowExW(
            std::ptr::null_mut(),
            hwnd,
            worker.as_ptr(),
            std::ptr::null(),
        );
        return 0;
    }
    1
}

/// Where a desktop-layer window goes: `(parent, insert_after)`. `insert_after` is null for
/// the classic WorkerW, `SHELLDLL_DefView` for the 24H2 Progman child.
fn desktop_parent() -> Option<(HWND, HWND)> {
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
            0x1,
            SMTO_NORMAL,
            1000,
            &mut result,
        );
        // 24H2+ (or no WorkerW spawned): DefView stays inside Progman → Progman child.
        let shelldll = wide("SHELLDLL_DefView");
        let defview = FindWindowExW(
            progman,
            std::ptr::null_mut(),
            shelldll.as_ptr(),
            std::ptr::null(),
        );
        if !defview.is_null() {
            return Some((progman, defview));
        }
        // Classic: 0x052C moved DefView into a top-level WorkerW and spawned the wallpaper
        // WorkerW right after it.
        let mut workerw: HWND = std::ptr::null_mut();
        EnumWindows(enum_windows_proc, &mut workerw as *mut HWND as isize);
        (!workerw.is_null()).then_some((workerw, std::ptr::null_mut()))
    }
}

/// The classic WorkerW HWND (null on 24H2+, see [`attach_to_desktop`]).
pub fn find_workerw() -> Option<isize> {
    match desktop_parent()? {
        (worker, after) if after.is_null() => Some(worker as isize),
        _ => None,
    }
}

fn reparent(hwnd: HWND) -> crate::Result<()> {
    let (parent, after) = desktop_parent()
        .ok_or_else(|| crate::Error::new("desktop layer: Progman / WorkerW not found"))?;
    unsafe {
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect);
        SetLastError(0);
        if SetParent(hwnd, parent).is_null() && GetLastError() != 0 {
            return Err(crate::Error::new(format!(
                "desktop layer: SetParent failed ({})",
                GetLastError()
            )));
        }
        // Child coordinates are relative to the parent (WorkerW / Progman span the virtual
        // screen, whose origin is not 0,0 with a monitor left of / above the primary).
        let mut pt = POINT {
            x: rect.left,
            y: rect.top,
        };
        ScreenToClient(parent, &mut pt);
        let z = if after.is_null() { SWP_NOZORDER } else { 0 };
        SetWindowPos(
            hwnd,
            after,
            pt.x,
            pt.y,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE | z,
        );
    }
    Ok(())
}

/// Parent `hwnd` (a top-level Tauri window, as `isize`) behind the desktop icons and keep
/// it there across Explorer restarts.
pub fn attach_to_desktop(hwnd: isize) -> crate::Result<()> {
    reparent(hwnd as HWND)?;
    {
        let mut list = ATTACHED.lock().unwrap();
        if !list.contains(&hwnd) {
            list.push(hwnd);
        }
    }
    WATCHER.call_once(|| {
        std::thread::spawn(watch_explorer);
    });
    Ok(())
}

/// Pre-0.8 name of [`attach_to_desktop`].
#[deprecated(note = "use attach_to_desktop")]
pub fn attach_to_workerw(child_hwnd: isize) -> bool {
    attach_to_desktop(child_hwnd).is_ok()
}

/// Convenience for a Tauri window.
pub fn attach_window<R: tauri::Runtime>(win: &tauri::WebviewWindow<R>) -> crate::Result<()> {
    let hwnd = win
        .hwnd()
        .map_err(|e| crate::Error::new(format!("desktop layer: hwnd: {e}")))?;
    attach_to_desktop(hwnd.0 as isize)
}

static TASKBAR_CREATED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

unsafe extern "system" fn watcher_proc(hwnd: HWND, msg: u32, w: usize, l: isize) -> isize {
    let tc = TASKBAR_CREATED.load(std::sync::atomic::Ordering::Relaxed);
    if tc != 0 && msg == tc {
        // Copy out: SetParent sends to the windows' (main) thread, which may be waiting
        // on this lock in `attach_to_desktop`.
        let live: Vec<isize> = {
            let mut list = ATTACHED.lock().unwrap();
            list.retain(|&h| IsWindow(h as HWND) != 0);
            list.clone()
        };
        for h in live {
            if let Err(e) = reparent(h as HWND) {
                log::warn!("desktop layer re-attach after Explorer restart: {e}");
            }
        }
        return 0;
    }
    DefWindowProcW(hwnd, msg, w, l)
}

/// Hidden top-level window: `TaskbarCreated` is broadcast to top-level windows only, and
/// attached widget windows are children now. Message-only windows get no broadcasts.
fn watch_explorer() {
    unsafe {
        TASKBAR_CREATED.store(
            RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()),
            std::sync::atomic::Ordering::Relaxed,
        );
        let class = wide("TauriWidgetsExplorerWatch");
        let instance = GetModuleHandleW(std::ptr::null());
        let wc = WNDCLASSW {
            style: 0,
            lpfn_wnd_proc: watcher_proc,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: instance,
            h_icon: std::ptr::null_mut(),
            h_cursor: std::ptr::null_mut(),
            hbr_background: std::ptr::null_mut(),
            lpsz_menu_name: std::ptr::null(),
            lpsz_class_name: class.as_ptr(),
        };
        RegisterClassW(&wc);
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class.as_ptr(),
            class.as_ptr(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            std::ptr::null_mut(),
        );
        if hwnd.is_null() {
            log::warn!(
                "desktop layer: Explorer watcher window failed ({})",
                GetLastError()
            );
            return;
        }
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            DispatchMessageW(&msg);
        }
    }
}
