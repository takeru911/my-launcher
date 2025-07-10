use crate::ui::alt_tab_grid::GridItem;
use crate::filter::Searchable;

/// ウィンドウ情報を保持する構造体
#[derive(Clone, Debug)]
pub struct WindowItem {
    pub hwnd: isize,
    pub title: String,
    pub process_name: String,
    pub class_name: String,
    pub is_visible: bool,
    pub is_minimized: bool,
    pub rect: (i32, i32, i32, i32), // (x, y, width, height)
}

impl WindowItem {
    pub fn new(
        hwnd: isize,
        title: String,
        process_name: String,
        class_name: String,
    ) -> Self {
        Self {
            hwnd,
            title,
            process_name,
            class_name,
            is_visible: true,
            is_minimized: false,
            rect: (0, 0, 0, 0),
        }
    }

    /// ウィンドウが有効かどうかをチェック
    pub fn is_valid(&self) -> bool {
        !self.title.trim().is_empty() && self.is_visible
    }
}

impl GridItem for WindowItem {
    fn title(&self) -> &str {
        &self.title
    }

    fn description(&self) -> &str {
        &self.process_name
    }

    fn hwnd(&self) -> isize {
        self.hwnd
    }

    fn id(&self) -> String {
        format!("window_{}", self.hwnd)
    }
}

impl Searchable for WindowItem {
    fn search_fields(&self) -> Vec<(&str, &str)> {
        vec![
            ("title", &self.title),
            ("process_name", &self.process_name),
            ("class_name", &self.class_name),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_window() -> WindowItem {
        WindowItem {
            hwnd: 12345,
            title: "Test Window - Notepad".to_string(),
            process_name: "notepad.exe".to_string(),
            class_name: "Notepad".to_string(),
            is_visible: true,
            is_minimized: false,
            rect: (100, 200, 800, 600),
        }
    }
    
    #[test]
    fn test_window_item_creation() {
        let window = WindowItem::new(
            100,
            "Test".to_string(),
            "test.exe".to_string(),
            "TestClass".to_string(),
        );
        
        assert_eq!(window.hwnd, 100);
        assert_eq!(window.title, "Test");
        assert_eq!(window.process_name, "test.exe");
        assert_eq!(window.class_name, "TestClass");
        assert!(window.is_visible);
        assert!(!window.is_minimized);
        assert_eq!(window.rect, (0, 0, 0, 0));
    }
    
    #[test]
    fn test_window_item_is_valid() {
        let mut window = create_test_window();
        
        // 正常なウィンドウ
        assert!(window.is_valid());
        
        // 非表示のウィンドウ
        window.is_visible = false;
        assert!(!window.is_valid());
        
        // タイトルが空のウィンドウ
        window.is_visible = true;
        window.title = "".to_string();
        assert!(!window.is_valid());
        
        // タイトルが空白のみのウィンドウ
        window.title = "   ".to_string();
        assert!(!window.is_valid());
    }
    
    #[test]
    fn test_grid_item_implementation() {
        let window = create_test_window();
        
        assert_eq!(window.title(), "Test Window - Notepad");
        assert_eq!(window.description(), "notepad.exe");
        assert_eq!(window.hwnd(), 12345);
        assert_eq!(window.id(), "window_12345");
    }
    
    #[test]
    fn test_searchable_implementation() {
        let window = create_test_window();
        let fields = window.search_fields();
        
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0], ("title", "Test Window - Notepad"));
        assert_eq!(fields[1], ("process_name", "notepad.exe"));
        assert_eq!(fields[2], ("class_name", "Notepad"));
    }
    
    #[test]
    fn test_window_item_clone() {
        let window1 = create_test_window();
        let window2 = window1.clone();
        
        assert_eq!(window1.hwnd, window2.hwnd);
        assert_eq!(window1.title, window2.title);
        assert_eq!(window1.process_name, window2.process_name);
    }
}