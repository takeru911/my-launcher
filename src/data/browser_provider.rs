use super::browser_item::{BookmarkItem, HistoryItem, ChromeBookmarks};
use std::path::PathBuf;
use std::fs;
use std::error::Error;

pub trait BrowserDataProvider: Send + Sync {
    fn get_bookmarks(&self) -> Result<Vec<BookmarkItem>, Box<dyn Error>>;
    fn get_history(&self) -> Result<Vec<HistoryItem>, Box<dyn Error>>;
    fn search_bookmarks(&self, query: &str) -> Result<Vec<BookmarkItem>, Box<dyn Error>>;
    fn search_history(&self, query: &str) -> Result<Vec<HistoryItem>, Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct BrowserProfile {
    pub browser_name: String,
    pub profile_name: String,
    pub profile_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub enable_chrome: bool,
    pub enable_wavebox: bool,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        // 環境変数から設定を読み取る
        let enable_chrome = std::env::var("LAUNCHER_ENABLE_CHROME")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);  // デフォルトでChromeは無効
            
        let enable_wavebox = std::env::var("LAUNCHER_ENABLE_WAVEBOX")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true);   // デフォルトでWaveboxは有効
            
        log::info!("Browser config: Chrome={}, Wavebox={}", enable_chrome, enable_wavebox);
            
        Self {
            enable_chrome,
            enable_wavebox,
        }
    }
}

pub struct ChromeBrowserProvider {
    profiles: Vec<BrowserProfile>,
    config: BrowserConfig,
}

impl ChromeBrowserProvider {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Self::with_config(BrowserConfig::default())
    }
    
    pub fn with_config(config: BrowserConfig) -> Result<Self, Box<dyn Error>> {
        let profiles = Self::find_all_profiles(&config);
        log::info!("Found {} browser profiles", profiles.len());
        for profile in &profiles {
            log::debug!("  {} - {}: {:?}", profile.browser_name, profile.profile_name, profile.profile_path);
        }
        Ok(Self { profiles, config })
    }

    fn find_all_profiles(config: &BrowserConfig) -> Vec<BrowserProfile> {
        let mut profiles = Vec::new();
        
        #[cfg(target_os = "windows")]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let base_path = PathBuf::from(&local_app_data);
                
                // Chrome profiles (if enabled)
                if config.enable_chrome {
                    let chrome_base = base_path.join("Google").join("Chrome").join("User Data");
                    if chrome_base.exists() {
                        profiles.extend(Self::find_profiles_in_directory(&chrome_base, "Chrome"));
                    }
                }
                
                // Wavebox profiles (if enabled)
                if config.enable_wavebox {
                    let wavebox_base = base_path.join("WaveboxApp").join("User Data");
                    if wavebox_base.exists() {
                        profiles.extend(Self::find_profiles_in_directory(&wavebox_base, "Wavebox"));
                    }
                }
            }
        }
        
        profiles
    }

    fn find_profiles_in_directory(base_path: &PathBuf, browser_name: &str) -> Vec<BrowserProfile> {
        let mut profiles = Vec::new();
        
        // Check Default profile
        let default_path = base_path.join("Default");
        if default_path.exists() && default_path.join("Bookmarks").exists() {
            profiles.push(BrowserProfile {
                browser_name: browser_name.to_string(),
                profile_name: "Default".to_string(),
                profile_path: default_path,
            });
        }
        
        // Check numbered profiles (Profile 1, Profile 2, etc.)
        for i in 1..10 {
            let profile_name = format!("Profile {}", i);
            let profile_path = base_path.join(&profile_name);
            if profile_path.exists() && profile_path.join("Bookmarks").exists() {
                profiles.push(BrowserProfile {
                    browser_name: browser_name.to_string(),
                    profile_name: profile_name,
                    profile_path: profile_path,
                });
            }
        }
        
        profiles
    }

    fn search_bookmarks_internal(&self, query: Option<&str>) -> Result<Vec<BookmarkItem>, Box<dyn Error>> {
        let mut all_bookmarks = Vec::new();
        
        for profile in &self.profiles {
            let bookmarks_path = profile.profile_path.join("Bookmarks");
            
            if !bookmarks_path.exists() {
                continue;
            }

            match fs::read_to_string(&bookmarks_path) {
                Ok(json_content) => {
                    match serde_json::from_str::<ChromeBookmarks>(&json_content) {
                        Ok(chrome_bookmarks) => {
                            // ブックマークにブラウザとプロファイル情報を付加
                            let profile_info = format!("{} - {}", profile.browser_name, profile.profile_name);
                            
                            for mut bookmark in chrome_bookmarks.roots.bookmark_bar.flatten(Some(&profile_info)) {
                                bookmark.browser_name = Some(profile.browser_name.clone());
                                bookmark.profile_name = Some(profile.profile_name.clone());
                                
                                // 空のタイトルは除外
                                if bookmark.title.is_empty() {
                                    continue;
                                }
                                
                                // クエリが指定されている場合はフィルタリング
                                if let Some(q) = query {
                                    let q_lower = q.to_lowercase();
                                    if bookmark.title.to_lowercase().contains(&q_lower) || 
                                       bookmark.url.to_lowercase().contains(&q_lower) ||
                                       profile.browser_name.to_lowercase().contains(&q_lower) ||
                                       profile.profile_name.to_lowercase().contains(&q_lower) {
                                        all_bookmarks.push(bookmark);
                                    }
                                } else {
                                    all_bookmarks.push(bookmark);
                                }
                            }
                            
                            for mut bookmark in chrome_bookmarks.roots.other.flatten(Some(&profile_info)) {
                                bookmark.browser_name = Some(profile.browser_name.clone());
                                bookmark.profile_name = Some(profile.profile_name.clone());
                                
                                // 空のタイトルは除外
                                if bookmark.title.is_empty() {
                                    continue;
                                }
                                
                                // クエリが指定されている場合はフィルタリング
                                if let Some(q) = query {
                                    let q_lower = q.to_lowercase();
                                    if bookmark.title.to_lowercase().contains(&q_lower) || 
                                       bookmark.url.to_lowercase().contains(&q_lower) ||
                                       profile.browser_name.to_lowercase().contains(&q_lower) ||
                                       profile.profile_name.to_lowercase().contains(&q_lower) {
                                        all_bookmarks.push(bookmark);
                                    }
                                } else {
                                    all_bookmarks.push(bookmark);
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to parse bookmarks for {} - {}: {}", 
                                profile.browser_name, profile.profile_name, e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to read bookmarks for {} - {}: {}", 
                        profile.browser_name, profile.profile_name, e);
                }
            }
        }
        
        Ok(all_bookmarks)
    }

    fn search_history_internal(&self, query: Option<&str>) -> Result<Vec<HistoryItem>, Box<dyn Error>> {
        let mut all_history = Vec::new();
        
        for profile in &self.profiles {
            let history_path = profile.profile_path.join("History");
            log::debug!("Attempting to read history from {} - {}: {:?}", 
                profile.browser_name, profile.profile_name, history_path);
            
            if !history_path.exists() {
                log::debug!("History file does not exist for {} - {}", 
                    profile.browser_name, profile.profile_name);
                continue;
            }

            // 直接読み取りを試みる
            match self.query_history_db(&history_path, &profile.browser_name, &profile.profile_name, query) {
                Ok(mut items) => {
                    log::info!("Successfully read {} history items from {} - {}", 
                        items.len(), profile.browser_name, profile.profile_name);
                    all_history.append(&mut items);
                }
                Err(e) => {
                    log::error!("Failed to read history for {} - {}: {}", 
                        profile.browser_name, profile.profile_name, e);
                }
            }
        }
        
        Ok(all_history)
    }

    #[cfg(feature = "sqlite")]
    fn query_history_db(&self, db_path: &PathBuf, browser_name: &str, profile_name: &str, query: Option<&str>) -> Result<Vec<HistoryItem>, Box<dyn Error>> {
        use rusqlite::Connection;
        
        log::debug!("Opening SQLite database at: {:?}", db_path);
        
        // イミュータブルモードでファイルを開く（ChromeがロックしていてもOK）
        let db_path_str = db_path.to_str().ok_or("Invalid path")?;
        let uri = format!("file:{}?mode=ro&immutable=1", db_path_str);
        
        let conn = Connection::open(&uri)?;
        log::debug!("Successfully opened SQLite connection with immutable mode");
        
        // クエリに基づいてSQL文とパラメータを準備
        // Chrome の last_visit_time は Webkit timestamp (1601年1月1日からのマイクロ秒)
        // 2週間前の timestamp を計算
        let two_weeks_ago = {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            // Unix timestamp を Webkit timestamp に変換 (1601年1月1日との差は11644473600秒)
            let webkit_now = (now + 11644473600) * 1_000_000; // マイクロ秒に変換
            webkit_now - (14 * 24 * 60 * 60 * 1_000_000) // 2週間前
        };
        
        let (sql, params): (&str, Vec<String>) = if let Some(q) = query {
            // SQLインジェクション対策のため、パラメータバインディングを使用
            (
                "SELECT url, title, visit_count, last_visit_time 
                 FROM urls 
                 WHERE title IS NOT NULL 
                   AND title != ''
                   AND last_visit_time > ?1
                   AND (LOWER(title) LIKE LOWER(?2) OR LOWER(url) LIKE LOWER(?2))
                 ORDER BY visit_count DESC, last_visit_time DESC 
                 LIMIT 100",
                vec![two_weeks_ago.to_string(), format!("%{}%", q)]
            )
        } else {
            (
                "SELECT url, title, visit_count, last_visit_time 
                 FROM urls 
                 WHERE title IS NOT NULL 
                   AND title != ''
                   AND last_visit_time > ?1
                 ORDER BY visit_count DESC, last_visit_time DESC 
                 LIMIT 100",
                vec![two_weeks_ago.to_string()]
            )
        };
        
        let mut stmt = conn.prepare(sql)?;
        
        // パラメータの有無に応じて適切な方法でクエリを実行
        let mut items = Vec::new();
        
        // パラメータをスライスに変換
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter()
            .map(|p| p as &dyn rusqlite::ToSql)
            .collect();
            
        let history_iter = stmt.query_map(&param_refs[..], |row| {
            Ok(HistoryItem {
                url: row.get(0)?,
                title: row.get(1)?,
                visit_count: row.get(2)?,
                last_visit_time: row.get(3)?,
                browser_name: Some(browser_name.to_string()),
                profile_name: Some(profile_name.to_string()),
            })
        })?;
        
        for item in history_iter {
            match item {
                Ok(history_item) => {
                    log::debug!("Found history item: {} - {}", history_item.title, history_item.url);
                    items.push(history_item);
                }
                Err(e) => {
                    log::error!("Error reading history item: {}", e);
                }
            }
        }
        
        log::info!("Successfully read {} history items", items.len());
        Ok(items)
    }

    #[cfg(not(feature = "sqlite"))]
    fn query_history_db(&self, _db_path: &PathBuf, _browser_name: &str, _profile_name: &str, _query: Option<&str>) -> Result<Vec<HistoryItem>, Box<dyn Error>> {
        // SQLite機能が無効な場合は空のベクタを返す
        log::warn!("SQLite feature is not enabled. History search is disabled.");
        Ok(Vec::new())
    }
}

impl BrowserDataProvider for ChromeBrowserProvider {
    fn get_bookmarks(&self) -> Result<Vec<BookmarkItem>, Box<dyn Error>> {
        self.search_bookmarks_internal(None)
    }

    fn get_history(&self) -> Result<Vec<HistoryItem>, Box<dyn Error>> {
        self.search_history_internal(None)
    }
    
    fn search_bookmarks(&self, query: &str) -> Result<Vec<BookmarkItem>, Box<dyn Error>> {
        self.search_bookmarks_internal(Some(query))
    }
    
    fn search_history(&self, query: &str) -> Result<Vec<HistoryItem>, Box<dyn Error>> {
        self.search_history_internal(Some(query))
    }
}

// キャッシュ付きプロバイダー
use std::sync::Mutex;

pub struct CachedBrowserProvider {
    inner: Box<dyn BrowserDataProvider>,
    bookmarks_cache: Mutex<Option<Vec<BookmarkItem>>>,
    history_cache: Mutex<Option<Vec<HistoryItem>>>,
}

impl CachedBrowserProvider {
    pub fn new(provider: Box<dyn BrowserDataProvider>) -> Self {
        Self {
            inner: provider,
            bookmarks_cache: Mutex::new(None),
            history_cache: Mutex::new(None),
        }
    }

    pub fn refresh(&mut self) {
        *self.bookmarks_cache.lock().unwrap() = None;
        *self.history_cache.lock().unwrap() = None;
    }
}

impl BrowserDataProvider for CachedBrowserProvider {
    fn get_bookmarks(&self) -> Result<Vec<BookmarkItem>, Box<dyn Error>> {
        let mut cache = self.bookmarks_cache.lock().unwrap();
        if let Some(cached) = cache.as_ref() {
            return Ok(cached.clone());
        }
        
        let bookmarks = self.inner.get_bookmarks()?;
        *cache = Some(bookmarks.clone());
        Ok(bookmarks)
    }

    fn get_history(&self) -> Result<Vec<HistoryItem>, Box<dyn Error>> {
        let mut cache = self.history_cache.lock().unwrap();
        if let Some(cached) = cache.as_ref() {
            return Ok(cached.clone());
        }
        
        let history = self.inner.get_history()?;
        *cache = Some(history.clone());
        Ok(history)
    }
    
    fn search_bookmarks(&self, query: &str) -> Result<Vec<BookmarkItem>, Box<dyn Error>> {
        // 検索はキャッシュせずに直接実行
        self.inner.search_bookmarks(query)
    }
    
    fn search_history(&self, query: &str) -> Result<Vec<HistoryItem>, Box<dyn Error>> {
        // 検索はキャッシュせずに直接実行
        self.inner.search_history(query)
    }
}