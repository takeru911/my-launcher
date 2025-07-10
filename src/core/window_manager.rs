#[derive(Debug, Clone, PartialEq)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub class_name: String,
    pub process_name: String,
    pub is_visible: bool,
    pub is_minimized: bool,
    pub rect: (i32, i32, i32, i32), // x, y, width, height
}

impl WindowInfo {
    pub fn contains_text(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.title.to_lowercase().contains(&query_lower)
            || self.process_name.to_lowercase().contains(&query_lower)
            || self.class_name.to_lowercase().contains(&query_lower)
    }
}

pub trait WindowManager: Send + Sync {
    fn enumerate_windows(&self) -> Vec<WindowInfo>;
    fn switch_to_window(&self, hwnd: isize);
}

#[cfg(windows)]
pub struct WindowsApiManager;

#[cfg(windows)]
impl WindowsApiManager {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(windows)]
impl WindowManager for WindowsApiManager {
    fn enumerate_windows(&self) -> Vec<WindowInfo> {
        crate::windows_api::enumerate_windows()
    }

    fn switch_to_window(&self, hwnd: isize) {
        crate::windows_api::switch_to_window(hwnd);
    }
}

#[cfg(not(windows))]
pub struct WindowsApiManager;

#[cfg(not(windows))]
impl WindowsApiManager {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(windows))]
impl WindowManager for WindowsApiManager {
    fn enumerate_windows(&self) -> Vec<WindowInfo> {
        vec![]
    }

    fn switch_to_window(&self, _hwnd: isize) {}
}

#[cfg(any(test, feature = "test-support"))]
pub mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    pub struct MockWindowManager {
        windows: Arc<Mutex<Vec<WindowInfo>>>,
        switched_to: Arc<Mutex<Option<isize>>>,
    }

    impl MockWindowManager {
        pub fn new(windows: Vec<WindowInfo>) -> Self {
            Self {
                windows: Arc::new(Mutex::new(windows)),
                switched_to: Arc::new(Mutex::new(None)),
            }
        }

        pub fn get_switched_window(&self) -> Option<isize> {
            *self.switched_to.lock().unwrap()
        }

        pub fn set_windows(&self, windows: Vec<WindowInfo>) {
            *self.windows.lock().unwrap() = windows;
        }
    }

    impl WindowManager for MockWindowManager {
        fn enumerate_windows(&self) -> Vec<WindowInfo> {
            self.windows.lock().unwrap().clone()
        }

        fn switch_to_window(&self, hwnd: isize) {
            *self.switched_to.lock().unwrap() = Some(hwnd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mock::MockWindowManager;

    #[test]
    fn test_window_info_contains_text() {
        let window = WindowInfo {
            hwnd: 1,
            title: "Visual Studio Code".to_string(),
            class_name: "Chrome_WidgetWin_1".to_string(),
            process_name: "Code.exe".to_string(),
            is_visible: true,
            is_minimized: false,
            rect: (0, 0, 1920, 1080),
        };

        assert!(window.contains_text("visual"));
        assert!(window.contains_text("STUDIO"));
        assert!(window.contains_text("code"));
        assert!(window.contains_text("Chrome"));
        assert!(window.contains_text(".exe"));
        assert!(!window.contains_text("notepad"));
    }

    #[test]
    fn test_mock_window_manager() {
        let windows = vec![
            WindowInfo {
                hwnd: 1,
                title: "Test Window".to_string(),
                class_name: "TestClass".to_string(),
                process_name: "test.exe".to_string(),
                is_visible: true,
                is_minimized: false,
                rect: (0, 0, 100, 100),
            },
        ];

        let manager = MockWindowManager::new(windows.clone());
        
        let enumerated = manager.enumerate_windows();
        assert_eq!(enumerated.len(), 1);
        assert_eq!(enumerated[0].title, "Test Window");

        manager.switch_to_window(123);
        assert_eq!(manager.get_switched_window(), Some(123));
    }
}