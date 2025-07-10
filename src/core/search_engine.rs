use super::window_manager::WindowInfo;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SearchMode {
    Browser,
    Windows,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    SwitchWindow(isize),
    GoogleSearch(String),
    OpenBookmark(String), // URL
    OpenHistory(String),  // URL
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResultType {
    GoogleSearch,
    Bookmark,
    History,
    Window,
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub title: String,
    pub description: String,
    pub action: Action,
    pub window_info: Option<WindowInfo>,
    pub result_type: ResultType,
}

pub trait SearchEngine {
    fn search(&self, query: &str, mode: SearchMode, windows: &[WindowInfo]) -> Vec<SearchResult>;
    fn is_window_search(&self, query: &str, mode: SearchMode) -> bool;
}

pub struct DefaultSearchEngine;

impl DefaultSearchEngine {
    pub fn new() -> Self {
        Self
    }
}

impl SearchEngine for DefaultSearchEngine {
    fn search(&self, query: &str, mode: SearchMode, windows: &[WindowInfo]) -> Vec<SearchResult> {
        let mut results = Vec::new();

        match mode {
            SearchMode::Browser => {
                if !query.is_empty() {
                    // Google検索を最初に追加
                    results.push(SearchResult {
                        title: format!("Google: {}", query),
                        description: "Search on Google".to_string(),
                        action: Action::GoogleSearch(query.to_string()),
                        window_info: None,
                        result_type: ResultType::GoogleSearch,
                    });
                    
                    // TODO: ブックマークと履歴の検索結果を追加
                    // ここでは後でブラウザプロバイダーを使用して実装
                }
            }
            SearchMode::Windows => {
                if query.is_empty() {
                    for window in windows {
                        results.push(SearchResult {
                            title: window.title.clone(),
                            description: format!("{} - {}", window.process_name, window.class_name),
                            action: Action::SwitchWindow(window.hwnd),
                            window_info: Some(window.clone()),
                            result_type: ResultType::Window,
                        });
                    }
                } else {
                    for window in windows {
                        if window.contains_text(query) {
                            results.push(SearchResult {
                                title: window.title.clone(),
                                description: format!("{} - {}", window.process_name, window.class_name),
                                action: Action::SwitchWindow(window.hwnd),
                                window_info: Some(window.clone()),
                                result_type: ResultType::Window,
                            });
                        }
                    }

                    if results.len() > 10 {
                        results.truncate(10);
                    }
                }
            }
        }

        results
    }

    fn is_window_search(&self, _query: &str, mode: SearchMode) -> bool {
        mode == SearchMode::Windows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_windows() -> Vec<WindowInfo> {
        vec![
            WindowInfo {
                hwnd: 1,
                title: "Visual Studio Code".to_string(),
                class_name: "Chrome_WidgetWin_1".to_string(),
                process_name: "Code.exe".to_string(),
                is_visible: true,
                is_minimized: false,
                rect: (0, 0, 1920, 1080),
            },
            WindowInfo {
                hwnd: 2,
                title: "Google Chrome".to_string(),
                class_name: "Chrome_WidgetWin_1".to_string(),
                process_name: "chrome.exe".to_string(),
                is_visible: true,
                is_minimized: false,
                rect: (0, 0, 1920, 1080),
            },
            WindowInfo {
                hwnd: 3,
                title: "Notepad".to_string(),
                class_name: "Notepad".to_string(),
                process_name: "notepad.exe".to_string(),
                is_visible: true,
                is_minimized: true,
                rect: (100, 100, 800, 600),
            },
        ]
    }

    #[test]
    fn test_empty_search_browser_mode() {
        let engine = DefaultSearchEngine::new();
        let windows = create_test_windows();
        let results = engine.search("", SearchMode::Browser, &windows);
        assert!(results.is_empty()); // Browserモードでは空クエリでは何も返さない
    }

    #[test]
    fn test_empty_search_windows_mode() {
        let engine = DefaultSearchEngine::new();
        let windows = create_test_windows();
        let results = engine.search("", SearchMode::Windows, &windows);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].title, "Visual Studio Code");
    }


    #[test]
    fn test_window_search_in_windows_mode() {
        let engine = DefaultSearchEngine::new();
        let windows = create_test_windows();
        
        let results = engine.search("code", SearchMode::Windows, &windows);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Visual Studio Code");
    }

    #[test]
    fn test_window_search_case_insensitive() {
        let engine = DefaultSearchEngine::new();
        let windows = create_test_windows();
        
        let results = engine.search("NOTEPAD", SearchMode::Windows, &windows);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Notepad");
    }

    #[test]
    fn test_window_search_by_process_name() {
        let engine = DefaultSearchEngine::new();
        let windows = create_test_windows();
        
        let results = engine.search("exe", SearchMode::Windows, &windows);
        assert_eq!(results.len(), 3); // All have .exe in process name
    }


    #[test]
    fn test_search_results_limit() {
        let engine = DefaultSearchEngine::new();
        let mut windows = Vec::new();
        
        // Create 15 windows
        for i in 0..15 {
            windows.push(WindowInfo {
                hwnd: i,
                title: format!("Window {}", i),
                class_name: "TestClass".to_string(),
                process_name: "test.exe".to_string(),
                is_visible: true,
                is_minimized: false,
                rect: (0, 0, 100, 100),
            });
        }
        
        let results = engine.search("Window", SearchMode::Windows, &windows);
        assert_eq!(results.len(), 10); // Should be limited to 10
    }

    #[test]
    fn test_browser_search_mode() {
        let engine = DefaultSearchEngine::new();
        let windows = create_test_windows();
        
        let results = engine.search("rust programming", SearchMode::Browser, &windows);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Google: rust programming");
        assert_eq!(results[0].description, "Search on Google");
        assert_eq!(results[0].action, Action::GoogleSearch("rust programming".to_string()));
        assert_eq!(results[0].result_type, ResultType::GoogleSearch);
        assert!(results[0].window_info.is_none());
    }

    #[test]
    fn test_browser_search_japanese() {
        let engine = DefaultSearchEngine::new();
        let windows = create_test_windows();
        
        let results = engine.search("日本語検索", SearchMode::Browser, &windows);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Google: 日本語検索");
        assert_eq!(results[0].action, Action::GoogleSearch("日本語検索".to_string()));
        assert_eq!(results[0].result_type, ResultType::GoogleSearch);
    }
}