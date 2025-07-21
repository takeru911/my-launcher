use crate::core::ChromeTab;
use crate::filter::Searchable;

#[derive(Debug, Clone)]
pub struct TabItem {
    pub tab: ChromeTab,
}

impl TabItem {
    pub fn new(tab: ChromeTab) -> Self {
        Self { tab }
    }
    
    pub fn get_display_name(&self) -> String {
        if self.tab.title.is_empty() {
            self.tab.url.clone()
        } else {
            self.tab.title.clone()
        }
    }
    
    pub fn get_url(&self) -> &str {
        &self.tab.url
    }
    
    pub fn get_id(&self) -> i32 {
        self.tab.id
    }
    
    pub fn get_window_id(&self) -> i32 {
        self.tab.window_id
    }
    
    pub fn is_active(&self) -> bool {
        self.tab.active
    }
}

impl Searchable for TabItem {
    fn search_fields(&self) -> Vec<(&str, &str)> {
        vec![
            ("title", &self.tab.title),
            ("url", &self.tab.url),
        ]
    }
}