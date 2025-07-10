use crate::filter::search_filter::Searchable;
use serde::{Deserialize, Serialize};

/// ブックマークアイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkItem {
    pub title: String,
    pub url: String,
    pub folder: Option<String>,
    pub browser_name: Option<String>,
    pub profile_name: Option<String>,
}

impl Searchable for BookmarkItem {
    fn search_fields(&self) -> Vec<(&str, &str)> {
        let mut fields = vec![
            ("title", self.title.as_str()),
            ("url", self.url.as_str()),
        ];
        
        // ブラウザ名とプロファイル名も検索対象に含める
        if let Some(browser) = &self.browser_name {
            fields.push(("browser", browser.as_str()));
        }
        if let Some(profile) = &self.profile_name {
            fields.push(("profile", profile.as_str()));
        }
        
        fields
    }
}

/// 履歴アイテム
#[derive(Debug, Clone)]
pub struct HistoryItem {
    pub title: String,
    pub url: String,
    pub visit_count: i32,
    pub last_visit_time: i64,
    pub browser_name: Option<String>,
    pub profile_name: Option<String>,
}

impl Searchable for HistoryItem {
    fn search_fields(&self) -> Vec<(&str, &str)> {
        let mut fields = vec![
            ("title", self.title.as_str()),
            ("url", self.url.as_str()),
        ];
        
        // ブラウザ名とプロファイル名も検索対象に含める
        if let Some(browser) = &self.browser_name {
            fields.push(("browser", browser.as_str()));
        }
        if let Some(profile) = &self.profile_name {
            fields.push(("profile", profile.as_str()));
        }
        
        fields
    }
}

/// Chrome Bookmarks JSON構造
#[derive(Debug, Deserialize)]
pub struct ChromeBookmarks {
    pub roots: BookmarkRoots,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkRoots {
    pub bookmark_bar: BookmarkNode,
    pub other: BookmarkNode,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkNode {
    #[serde(default)]
    pub children: Vec<BookmarkNode>,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub url: Option<String>,
}

impl BookmarkNode {
    pub fn flatten(&self, parent_folder: Option<&str>) -> Vec<BookmarkItem> {
        let mut items = Vec::new();
        
        if self.node_type == "url" {
            if let Some(url) = &self.url {
                items.push(BookmarkItem {
                    title: self.name.clone(),
                    url: url.clone(),
                    folder: parent_folder.map(|s| s.to_string()),
                    browser_name: None,
                    profile_name: None,
                });
            }
        } else if self.node_type == "folder" {
            let folder_name = if let Some(parent) = parent_folder {
                format!("{} > {}", parent, self.name)
            } else {
                self.name.clone()
            };
            
            for child in &self.children {
                items.extend(child.flatten(Some(&folder_name)));
            }
        }
        
        items
    }
}