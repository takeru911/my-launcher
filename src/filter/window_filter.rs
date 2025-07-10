use crate::data::window_item::WindowItem;

/// ウィンドウフィルタリングのトレイト
pub trait WindowFilter {
    /// ウィンドウがフィルタ条件に合致するかチェック
    fn matches(&self, window: &WindowItem) -> bool;
    
    /// フィルタ名を取得
    fn name(&self) -> &str;
}

/// タスクバーに表示されるウィンドウのフィルタ
pub struct TaskbarWindowFilter;

impl TaskbarWindowFilter {
    pub fn new() -> Self {
        Self
    }
    
    /// タスクバーに表示されるウィンドウかチェック
    #[cfg(windows)]
    fn is_taskbar_window(window: &WindowItem) -> bool {
        use winapi::um::winuser::{GetWindowLongW, GetWindow, GWL_EXSTYLE, GWL_STYLE, GW_OWNER};
        use winapi::um::winuser::{WS_VISIBLE};
        
        const WS_EX_TOOLWINDOW: u32 = 0x00000080;
        const WS_EX_APPWINDOW: u32 = 0x00040000;
        
        unsafe {
            let hwnd = window.hwnd as winapi::shared::windef::HWND;
            
            // ウィンドウスタイルを取得
            let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
            
            // 表示されていないウィンドウは除外
            if style & WS_VISIBLE == 0 {
                return false;
            }
            
            // タスクバーに表示される条件：
            // 1. WS_EX_APPWINDOW が設定されている
            // 2. または、WS_EX_TOOLWINDOW が設定されておらず、オーナーウィンドウがない
            let has_app_window = ex_style & WS_EX_APPWINDOW != 0;
            let is_tool_window = ex_style & WS_EX_TOOLWINDOW != 0;
            
            // オーナーウィンドウの確認
            let owner = GetWindow(hwnd, GW_OWNER);
            let has_no_owner = owner.is_null();
            
            // タスクバー表示の判定
            has_app_window || (!is_tool_window && has_no_owner)
        }
    }
    
    #[cfg(not(windows))]
    fn is_taskbar_window(window: &WindowItem) -> bool {
        // Windows以外の環境では基本的なチェックのみ
        window.is_visible && !window.title.trim().is_empty()
    }
}

impl WindowFilter for TaskbarWindowFilter {
    fn matches(&self, window: &WindowItem) -> bool {
        Self::is_taskbar_window(window)
    }
    
    fn name(&self) -> &str {
        "TaskbarWindowFilter"
    }
}

/// フィルタを組み合わせるモード
#[derive(Debug, Clone, Copy)]
pub enum FilterMode {
    /// すべてのフィルタに合致（AND）
    All,
    /// いずれかのフィルタに合致（OR）
    Any,
}

/// 複数のフィルタを組み合わせるフィルタ
pub struct CompositeFilter {
    filters: Vec<Box<dyn WindowFilter>>,
    mode: FilterMode,
}

impl CompositeFilter {
    pub fn new(mode: FilterMode) -> Self {
        Self {
            filters: Vec::new(),
            mode,
        }
    }
    
    pub fn add_filter(mut self, filter: Box<dyn WindowFilter>) -> Self {
        self.filters.push(filter);
        self
    }
}

impl WindowFilter for CompositeFilter {
    fn matches(&self, window: &WindowItem) -> bool {
        match self.mode {
            FilterMode::All => self.filters.iter().all(|f| f.matches(window)),
            FilterMode::Any => self.filters.iter().any(|f| f.matches(window)),
        }
    }
    
    fn name(&self) -> &str {
        "CompositeFilter"
    }
}

/// ウィンドウリストをフィルタリング
pub fn filter_windows(windows: Vec<WindowItem>, filter: &dyn WindowFilter) -> Vec<WindowItem> {
    windows.into_iter()
        .filter(|w| filter.matches(w))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::window_item::WindowItem;

    #[test]
    fn test_taskbar_filter_basic() {
        // Windows以外の環境でのテスト
        #[cfg(not(windows))]
        {
            let filter = TaskbarWindowFilter::new();
            
            // 表示されていて、タイトルがあるウィンドウは通過
            let window1 = WindowItem {
                hwnd: 1,
                title: "Test Window".to_string(),
                process_name: "test.exe".to_string(),
                class_name: "TestClass".to_string(),
                is_visible: true,
                is_minimized: false,
                rect: (0, 0, 100, 100),
            };
            assert!(filter.matches(&window1));
            
            // 表示されていないウィンドウは除外
            let window2 = WindowItem {
                hwnd: 2,
                title: "Hidden Window".to_string(),
                process_name: "test.exe".to_string(),
                class_name: "TestClass".to_string(),
                is_visible: false,
                is_minimized: false,
                rect: (0, 0, 100, 100),
            };
            assert!(!filter.matches(&window2));
            
            // タイトルが空のウィンドウは除外
            let window3 = WindowItem {
                hwnd: 3,
                title: "".to_string(),
                process_name: "test.exe".to_string(),
                class_name: "TestClass".to_string(),
                is_visible: true,
                is_minimized: false,
                rect: (0, 0, 100, 100),
            };
            assert!(!filter.matches(&window3));
        }
    }

    #[test]
    fn test_composite_filter_all_mode() {
        let filter1 = TaskbarWindowFilter::new();
        let filter2 = TaskbarWindowFilter::new(); // 実際には異なるフィルタを使うが、テスト用
        
        let composite = CompositeFilter::new(FilterMode::All)
            .add_filter(Box::new(filter1))
            .add_filter(Box::new(filter2));
        
        let window = WindowItem {
            hwnd: 1,
            title: "Test Window".to_string(),
            process_name: "test.exe".to_string(),
            class_name: "TestClass".to_string(),
            is_visible: true,
            is_minimized: false,
            rect: (0, 0, 100, 100),
        };
        
        #[cfg(not(windows))]
        assert!(composite.matches(&window));
    }

    #[test]
    fn test_composite_filter_any_mode() {
        // カスタムフィルタを作成
        struct AlwaysTrueFilter;
        impl WindowFilter for AlwaysTrueFilter {
            fn matches(&self, _: &WindowItem) -> bool { true }
            fn name(&self) -> &str { "AlwaysTrue" }
        }
        
        struct AlwaysFalseFilter;
        impl WindowFilter for AlwaysFalseFilter {
            fn matches(&self, _: &WindowItem) -> bool { false }
            fn name(&self) -> &str { "AlwaysFalse" }
        }
        
        let composite = CompositeFilter::new(FilterMode::Any)
            .add_filter(Box::new(AlwaysFalseFilter))
            .add_filter(Box::new(AlwaysTrueFilter));
        
        let window = WindowItem {
            hwnd: 1,
            title: "Test".to_string(),
            process_name: "test.exe".to_string(),
            class_name: "TestClass".to_string(),
            is_visible: true,
            is_minimized: false,
            rect: (0, 0, 100, 100),
        };
        
        assert!(composite.matches(&window));
    }

    #[test]
    fn test_filter_windows_function() {
        let windows = vec![
            WindowItem {
                hwnd: 1,
                title: "Visible Window".to_string(),
                process_name: "app.exe".to_string(),
                class_name: "AppClass".to_string(),
                is_visible: true,
                is_minimized: false,
                rect: (0, 0, 100, 100),
            },
            WindowItem {
                hwnd: 2,
                title: "".to_string(),
                process_name: "hidden.exe".to_string(),
                class_name: "HiddenClass".to_string(),
                is_visible: true,
                is_minimized: false,
                rect: (0, 0, 100, 100),
            },
        ];
        
        let filter = TaskbarWindowFilter::new();
        let filtered = filter_windows(windows, &filter);
        
        #[cfg(not(windows))]
        {
            assert_eq!(filtered.len(), 1);
            assert_eq!(filtered[0].title, "Visible Window");
        }
    }
}