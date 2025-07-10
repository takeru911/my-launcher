use super::window_item::WindowItem;
use crate::core::WindowInfo;

/// ウィンドウ情報を提供するトレイト
pub trait WindowProvider {
    /// 現在のウィンドウ一覧を取得
    fn get_windows(&self) -> Vec<WindowItem>;
    
    /// ウィンドウ一覧を更新
    fn refresh(&mut self);
    
    /// 特定のウィンドウにフォーカス
    fn focus_window(&self, hwnd: isize) -> Result<(), String>;
}

/// Windows API を使用したウィンドウプロバイダー
pub struct WindowsApiProvider {
    cached_windows: Vec<WindowItem>,
}

impl WindowsApiProvider {
    pub fn new() -> Self {
        Self {
            cached_windows: Vec::new(),
        }
    }

    /// WindowInfo から WindowItem への変換
    fn convert_window_info(&self, info: &WindowInfo) -> WindowItem {
        WindowItem {
            hwnd: info.hwnd,
            title: info.title.clone(),
            process_name: info.process_name.clone(),
            class_name: info.class_name.clone(),
            is_visible: info.is_visible,
            is_minimized: info.is_minimized,
            rect: info.rect,
        }
    }
}

impl WindowProvider for WindowsApiProvider {
    fn get_windows(&self) -> Vec<WindowItem> {
        self.cached_windows.clone()
    }

    fn refresh(&mut self) {
        #[cfg(windows)]
        {
            let windows = crate::windows_api::enumerate_windows();
            self.cached_windows = windows
                .into_iter()
                .map(|w| self.convert_window_info(&w))
                .filter(|w| w.is_valid())
                .collect();
            
            log::debug!("Refreshed windows: {} items", self.cached_windows.len());
        }

        #[cfg(not(windows))]
        {
            // テスト用のダミーデータ
            self.cached_windows = vec![
                WindowItem::new(1, "Test Window 1".to_string(), "test.exe".to_string(), "TestClass".to_string()),
                WindowItem::new(2, "Test Window 2".to_string(), "test2.exe".to_string(), "TestClass".to_string()),
                WindowItem::new(3, "Test Window 3".to_string(), "test3.exe".to_string(), "TestClass".to_string()),
            ];
        }
    }
    

    fn focus_window(&self, hwnd: isize) -> Result<(), String> {
        #[cfg(windows)]
        {
            crate::windows_api::switch_to_window(hwnd);
            Ok(())
        }

        #[cfg(not(windows))]
        {
            log::info!("Would switch to window: {}", hwnd);
            Ok(())
        }
    }
}

impl Default for WindowsApiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // モック用のWindowProvider実装
    struct MockWindowProvider {
        windows: Vec<WindowItem>,
        focus_called: std::cell::RefCell<Vec<isize>>,
    }
    
    impl MockWindowProvider {
        fn new(windows: Vec<WindowItem>) -> Self {
            Self {
                windows,
                focus_called: std::cell::RefCell::new(Vec::new()),
            }
        }
    }
    
    impl WindowProvider for MockWindowProvider {
        fn get_windows(&self) -> Vec<WindowItem> {
            self.windows.clone()
        }
        
        fn refresh(&mut self) {
            // テスト用なので何もしない
        }
        
        fn focus_window(&self, hwnd: isize) -> Result<(), String> {
            self.focus_called.borrow_mut().push(hwnd);
            Ok(())
        }
    }
    
    #[test]
    fn test_window_provider_trait() {
        let windows = vec![
            WindowItem::new(1, "Window 1".to_string(), "app1.exe".to_string(), "Class1".to_string()),
            WindowItem::new(2, "Window 2".to_string(), "app2.exe".to_string(), "Class2".to_string()),
        ];
        
        let provider = MockWindowProvider::new(windows.clone());
        
        // get_windows のテスト
        let retrieved = provider.get_windows();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].title, "Window 1");
        assert_eq!(retrieved[1].title, "Window 2");
        
        // focus_window のテスト
        assert!(provider.focus_window(1).is_ok());
        assert!(provider.focus_window(2).is_ok());
        
        let called = provider.focus_called.borrow();
        assert_eq!(*called, vec![1, 2]);
    }
    
    #[test]
    fn test_windows_api_provider_creation() {
        let provider = WindowsApiProvider::new();
        assert_eq!(provider.cached_windows.len(), 0);
    }
    
    #[test]
    fn test_windows_api_provider_convert_window_info() {
        let provider = WindowsApiProvider::new();
        let info = WindowInfo {
            hwnd: 12345,
            title: "Test Window".to_string(),
            class_name: "TestClass".to_string(),
            process_name: "test.exe".to_string(),
            is_visible: true,
            is_minimized: false,
            rect: (10, 20, 300, 400),
        };
        
        let item = provider.convert_window_info(&info);
        
        assert_eq!(item.hwnd, 12345);
        assert_eq!(item.title, "Test Window");
        assert_eq!(item.class_name, "TestClass");
        assert_eq!(item.process_name, "test.exe");
        assert_eq!(item.is_visible, true);
        assert_eq!(item.is_minimized, false);
        assert_eq!(item.rect, (10, 20, 300, 400));
    }
    
    #[cfg(not(windows))]
    #[test]
    fn test_windows_api_provider_refresh_non_windows() {
        let mut provider = WindowsApiProvider::new();
        provider.refresh();
        
        let windows = provider.get_windows();
        assert_eq!(windows.len(), 3); // ダミーデータが3つ
        assert_eq!(windows[0].title, "Test Window 1");
        assert_eq!(windows[1].title, "Test Window 2");
        assert_eq!(windows[2].title, "Test Window 3");
    }
}