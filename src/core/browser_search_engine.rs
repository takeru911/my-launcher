use super::{SearchEngine, SearchResult, SearchMode, Action, ResultType, window_manager::WindowInfo};
use crate::data::{
    browser_provider::{BrowserDataProvider, ChromeBrowserProvider, CachedBrowserProvider},
    browser_item::{BookmarkItem, HistoryItem},
};
use std::sync::{Arc, Mutex};

pub struct BrowserSearchEngine {
    browser_provider: Arc<Mutex<CachedBrowserProvider>>,
}

impl BrowserSearchEngine {
    pub fn new() -> Self {
        // Chrome用のプロバイダーを作成
        let chrome_provider = match ChromeBrowserProvider::new() {
            Ok(provider) => Box::new(provider) as Box<dyn BrowserDataProvider>,
            Err(e) => {
                log::error!("Failed to create Chrome provider: {}", e);
                // ダミープロバイダーを返す
                Box::new(DummyBrowserProvider)
            }
        };
        
        let cached_provider = CachedBrowserProvider::new(chrome_provider);
        
        Self {
            browser_provider: Arc::new(Mutex::new(cached_provider)),
        }
    }
    
    pub fn refresh_browser_data(&self) {
        if let Ok(mut provider) = self.browser_provider.lock() {
            provider.refresh();
        }
    }
}

impl SearchEngine for BrowserSearchEngine {
    fn search(&self, query: &str, mode: SearchMode, windows: &[WindowInfo]) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        match mode {
            SearchMode::Browser => {
                if !query.is_empty() {
                    // 1. Google検索を最初に追加
                    results.push(SearchResult {
                        title: format!("Google: {}", query),
                        description: "Search on Google".to_string(),
                        action: Action::GoogleSearch(query.to_string()),
                        window_info: None,
                        result_type: ResultType::GoogleSearch,
                    });
                    
                    // 2. ブックマークを検索
                    if let Ok(provider) = self.browser_provider.lock() {
                        match provider.search_bookmarks(query) {
                            Ok(bookmarks) => {
                                // すべてのブックマークを追加（UI側で表示制御）
                                for bookmark in bookmarks {
                                    // タイトルにブラウザとプロファイル情報を含める
                                    let title = if let (Some(browser), Some(profile)) = (&bookmark.browser_name, &bookmark.profile_name) {
                                        format!("[{} - {}] {}", browser, profile, bookmark.title)
                                    } else {
                                        bookmark.title.clone()
                                    };
                                    
                                    let mut description_parts = vec![bookmark.url.clone()];
                                    if let Some(folder) = &bookmark.folder {
                                        description_parts.push(folder.clone());
                                    }
                                    
                                    let description = description_parts.join(" | ");
                                    
                                    results.push(SearchResult {
                                        title,
                                        description,
                                        action: Action::OpenBookmark(bookmark.url),
                                        window_info: None,
                                        result_type: ResultType::Bookmark,
                                    });
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to get bookmarks: {}", e);
                            }
                        }
                        
                        // 3. 履歴を検索
                        match provider.search_history(query) {
                            Ok(history_items) => {
                                // すべての履歴を追加（UI側で表示制御）
                                for history in history_items {
                                    // タイトルにブラウザとプロファイル情報を含める
                                    let title = if let (Some(browser), Some(profile)) = (&history.browser_name, &history.profile_name) {
                                        format!("[{} - {}] {}", browser, profile, history.title)
                                    } else {
                                        history.title.clone()
                                    };
                                    
                                    // Webkit timestamp (microseconds since 1601-01-01) を日時に変換
                                    let last_visit_str = {
                                        use std::time::{SystemTime, UNIX_EPOCH};
                                        // Webkit timestamp を Unix timestamp に変換
                                        let webkit_epoch_diff = 11644473600_u64; // 1601-01-01 と 1970-01-01 の差（秒）
                                        let unix_timestamp_secs = (history.last_visit_time / 1_000_000) as u64;
                                        
                                        if unix_timestamp_secs > webkit_epoch_diff {
                                            let unix_secs = unix_timestamp_secs - webkit_epoch_diff;
                                            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                                            
                                            if unix_secs <= now {
                                                let elapsed_secs = now - unix_secs;
                                                if elapsed_secs < 60 {
                                                    "just now".to_string()
                                                } else if elapsed_secs < 3600 {
                                                    format!("{} min ago", elapsed_secs / 60)
                                                } else if elapsed_secs < 86400 {
                                                    format!("{} hours ago", elapsed_secs / 3600)
                                                } else if elapsed_secs < 604800 {
                                                    format!("{} days ago", elapsed_secs / 86400)
                                                } else {
                                                    format!("{} weeks ago", elapsed_secs / 604800)
                                                }
                                            } else {
                                                "future".to_string()
                                            }
                                        } else {
                                            "unknown".to_string()
                                        }
                                    };
                                    
                                    let description = format!("{} (visited {} times, {})", 
                                        history.url, 
                                        history.visit_count,
                                        last_visit_str
                                    );
                                    
                                    results.push(SearchResult {
                                        title,
                                        description,
                                        action: Action::OpenHistory(history.url),
                                        window_info: None,
                                        result_type: ResultType::History,
                                    });
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to get history: {}", e);
                            }
                        }
                    }
                }
            }
            SearchMode::Windows => {
                // Windowsモードは通常のウィンドウ検索
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


// ダミーのブラウザプロバイダー（Chrome情報が取得できない場合用）
struct DummyBrowserProvider;

impl BrowserDataProvider for DummyBrowserProvider {
    fn get_bookmarks(&self) -> Result<Vec<BookmarkItem>, Box<dyn std::error::Error>> {
        Ok(Vec::new())
    }
    
    fn get_history(&self) -> Result<Vec<HistoryItem>, Box<dyn std::error::Error>> {
        Ok(Vec::new())
    }
    
    fn search_bookmarks(&self, _query: &str) -> Result<Vec<BookmarkItem>, Box<dyn std::error::Error>> {
        Ok(Vec::new())
    }
    
    fn search_history(&self, _query: &str) -> Result<Vec<HistoryItem>, Box<dyn std::error::Error>> {
        Ok(Vec::new())
    }
}