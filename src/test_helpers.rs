#[cfg(test)]
pub mod helpers {
    use crate::core::{WindowInfo, SearchResult, Action};
    
    pub fn create_test_window(hwnd: isize, title: &str, process: &str) -> WindowInfo {
        WindowInfo {
            hwnd,
            title: title.to_string(),
            class_name: "TestClass".to_string(),
            process_name: process.to_string(),
            is_visible: true,
            is_minimized: false,
            rect: (0, 0, 1920, 1080),
        }
    }
    
    pub fn assert_search_result_contains_action(results: &[SearchResult], action: &Action) -> bool {
        results.iter().any(|r| &r.action == action)
    }
    
    pub fn assert_search_result_contains_title(results: &[SearchResult], title: &str) -> bool {
        results.iter().any(|r| r.title.contains(title))
    }
}