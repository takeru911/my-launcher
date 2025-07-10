#[cfg(windows)]
use winapi::{
    shared::{
        minwindef::{BOOL, LPARAM, TRUE},
        windef::{HWND, RECT},
    },
    um::{
        dwmapi::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
        processthreadsapi::OpenProcess,
        psapi::GetModuleFileNameExW,
        winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        winuser::{
            EnumWindows, GetClassNameW, GetWindowLongPtrW, GetWindowTextW, GetWindowThreadProcessId,
            IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow, GWL_STYLE, GWL_EXSTYLE, SW_RESTORE,
            WS_EX_TOOLWINDOW, WS_EX_APPWINDOW, GetWindow, GW_OWNER, WS_VISIBLE,
        },
    },
};
#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;
#[cfg(windows)]
use std::ptr;

use crate::core::WindowInfo;

#[cfg(windows)]
pub fn enumerate_windows() -> Vec<WindowInfo> {
    unsafe {
        let mut windows = Vec::new();
        let windows_ptr = &mut windows as *mut Vec<WindowInfo>;
        
        log::debug!("Starting window enumeration...");
        EnumWindows(Some(enum_window_callback), windows_ptr as LPARAM);
        log::debug!("Found {} windows after enumeration", windows.len());
        
        // No additional filtering needed - is_taskbar_window already filtered
        windows
    }
}

#[cfg(windows)]
unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam as *mut Vec<WindowInfo>);
    
    // Check if window is visible
    if IsWindowVisible(hwnd) == 0 {
        return TRUE;
    }
    
    // Check if this is a main window that should appear in taskbar
    if !is_taskbar_window(hwnd) {
        return TRUE;
    }
    
    let mut title = [0u16; 256];
    let title_len = GetWindowTextW(hwnd, title.as_mut_ptr(), 256);
    let title = OsString::from_wide(&title[..title_len as usize])
        .to_string_lossy()
        .to_string();
    
    if title.is_empty() {
        return TRUE;
    }
    
    let mut class_name = [0u16; 256];
    let class_len = GetClassNameW(hwnd, class_name.as_mut_ptr(), 256);
    let class_name = OsString::from_wide(&class_name[..class_len as usize])
        .to_string_lossy()
        .to_string();
    
    let mut process_id = 0;
    GetWindowThreadProcessId(hwnd, &mut process_id);
    
    let process_name = get_process_name(process_id).unwrap_or_default();
    
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    
    DwmGetWindowAttribute(
        hwnd,
        DWMWA_EXTENDED_FRAME_BOUNDS,
        &mut rect as *mut _ as *mut _,
        std::mem::size_of::<RECT>() as u32,
    );
    
    let window_info = WindowInfo {
        hwnd: hwnd as isize,
        title,
        class_name,
        process_name,
        is_visible: true,
        is_minimized: IsIconic(hwnd) == TRUE,
        rect: (
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
        ),
    };
    
    windows.push(window_info);
    TRUE
}

#[cfg(windows)]
unsafe fn get_process_name(process_id: u32) -> Option<String> {
    let process_handle = OpenProcess(
        PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
        0,
        process_id,
    );
    
    if process_handle.is_null() {
        return None;
    }
    
    let mut filename = [0u16; 1024];
    let len = GetModuleFileNameExW(
        process_handle,
        ptr::null_mut(),
        filename.as_mut_ptr(),
        1024,
    );
    
    winapi::um::handleapi::CloseHandle(process_handle);
    
    if len == 0 {
        return None;
    }
    
    let path = OsString::from_wide(&filename[..len as usize])
        .to_string_lossy()
        .to_string();
    
    path.split('\\').last().map(|s| s.to_string())
}

#[cfg(windows)]
pub fn switch_to_window(hwnd: isize) {
    unsafe {
        let hwnd = hwnd as HWND;
        
        if IsIconic(hwnd) == TRUE {
            ShowWindow(hwnd, SW_RESTORE);
        }
        
        SetForegroundWindow(hwnd);
    }
}

#[cfg(not(windows))]
pub fn enumerate_windows() -> Vec<WindowInfo> {
    vec![]
}

#[cfg(not(windows))]
pub fn switch_to_window(_hwnd: isize) {}

#[cfg(windows)]
unsafe fn is_taskbar_window(hwnd: HWND) -> bool {
    // Get window styles
    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
    
    // Skip tool windows
    if (ex_style & WS_EX_TOOLWINDOW) != 0 {
        return false;
    }
    
    // Skip windows without a title
    let mut title = [0u16; 256];
    let title_len = GetWindowTextW(hwnd, title.as_mut_ptr(), 256);
    if title_len == 0 {
        return false;
    }
    
    // Check if the window has an owner
    let owner = GetWindow(hwnd, GW_OWNER);
    
    // Windows that appear in the taskbar typically:
    // 1. Are visible (already checked)
    // 2. Don't have WS_EX_TOOLWINDOW style
    // 3. Either have WS_EX_APPWINDOW style OR have no owner
    // 4. Are not minimized to tray (still have WS_VISIBLE even when minimized)
    
    let has_appwindow = (ex_style & WS_EX_APPWINDOW) != 0;
    let has_no_owner = owner.is_null() || owner == hwnd;
    let is_visible = (style & WS_VISIBLE) != 0;
    
    // This window should appear in taskbar if:
    // - It's visible AND
    // - It either has WS_EX_APPWINDOW OR has no owner
    is_visible && (has_appwindow || has_no_owner)
}