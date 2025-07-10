use super::{SearchEngine, SearchResult, SearchMode, Action, WindowManager, WindowInfo};
use std::sync::Arc;

pub struct LauncherCore<S: SearchEngine, W: WindowManager> {
    search_engine: S,
    window_manager: Arc<W>,
    cached_windows: Vec<WindowInfo>,
}

impl<S: SearchEngine, W: WindowManager> LauncherCore<S, W> {
    pub fn new(search_engine: S, window_manager: Arc<W>) -> Self {
        let mut core = Self {
            search_engine,
            window_manager,
            cached_windows: Vec::new(),
        };
        core.refresh_windows();
        core
    }

    pub fn refresh_windows(&mut self) {
        self.cached_windows = self.window_manager.enumerate_windows();
    }

    pub fn search(&self, query: &str, mode: SearchMode) -> Vec<SearchResult> {
        self.search_engine.search(query, mode, &self.cached_windows)
    }

    pub fn execute_action(&self, action: &Action) {
        match action {
            Action::SwitchWindow(hwnd) => {
                self.window_manager.switch_to_window(*hwnd);
            }
            Action::GoogleSearch(query) => {
                let encoded_query = urlencoding::encode(query);
                let url = format!("https://www.google.com/search?q={}", encoded_query);
                let _ = open::that(&url);
            }
            Action::OpenBookmark(url) | Action::OpenHistory(url) => {
                let _ = open::that(url);
            }
        }
    }

    pub fn get_cached_windows(&self) -> &[WindowInfo] {
        &self.cached_windows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::search_engine::DefaultSearchEngine;
    use crate::core::window_manager::mock::MockWindowManager;

    fn create_test_launcher() -> (LauncherCore<DefaultSearchEngine, MockWindowManager>, Arc<MockWindowManager>) {
        let windows = vec![
            WindowInfo {
                hwnd: 1,
                title: "Test Editor".to_string(),
                class_name: "EditorClass".to_string(),
                process_name: "editor.exe".to_string(),
                is_visible: true,
                is_minimized: false,
                rect: (0, 0, 800, 600),
            },
            WindowInfo {
                hwnd: 2,
                title: "Browser".to_string(),
                class_name: "BrowserClass".to_string(),
                process_name: "browser.exe".to_string(),
                is_visible: true,
                is_minimized: true,
                rect: (100, 100, 1024, 768),
            },
        ];

        let window_manager = Arc::new(MockWindowManager::new(windows));
        let search_engine = DefaultSearchEngine::new();
        let launcher = LauncherCore::new(search_engine, window_manager.clone());

        (launcher, window_manager)
    }

    #[test]
    fn test_launcher_initialization() {
        let (launcher, _) = create_test_launcher();
        assert_eq!(launcher.get_cached_windows().len(), 2);
    }

    #[test]
    fn test_launcher_search_browser_mode() {
        let (launcher, _) = create_test_launcher();
        
        let results = launcher.search("test", SearchMode::Browser);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Google: test");
        assert_eq!(results[0].action, Action::GoogleSearch("test".to_string()));
    }

    #[test]
    fn test_launcher_search_windows_mode() {
        let (launcher, _) = create_test_launcher();
        
        let results = launcher.search("editor", SearchMode::Windows);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Editor");
    }

    #[test]
    fn test_launcher_execute_switch_window() {
        let (launcher, window_manager) = create_test_launcher();
        
        launcher.execute_action(&Action::SwitchWindow(42));
        assert_eq!(window_manager.get_switched_window(), Some(42));
    }

    #[test]
    fn test_launcher_refresh_windows() {
        let (mut launcher, window_manager) = create_test_launcher();
        
        // Initially 2 windows
        assert_eq!(launcher.get_cached_windows().len(), 2);
        
        // Add a new window
        let new_windows = vec![
            WindowInfo {
                hwnd: 3,
                title: "New Window".to_string(),
                class_name: "NewClass".to_string(),
                process_name: "new.exe".to_string(),
                is_visible: true,
                is_minimized: false,
                rect: (0, 0, 640, 480),
            },
        ];
        window_manager.set_windows(new_windows);
        
        // Refresh and check
        launcher.refresh_windows();
        assert_eq!(launcher.get_cached_windows().len(), 1);
        assert_eq!(launcher.get_cached_windows()[0].title, "New Window");
    }

}