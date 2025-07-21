use crate::core::{TabManager, ChromeTab};
use crate::data::tab_item::TabItem;
use std::sync::Arc;

pub trait TabProvider: Send + Sync {
    fn get_tabs(&self) -> Vec<TabItem>;
    fn search_tabs(&self, query: &str) -> Vec<TabItem>;
    fn update_tabs(&self, tabs: Vec<ChromeTab>);
}

pub struct ChromeTabProvider {
    tab_manager: Arc<TabManager>,
}

impl ChromeTabProvider {
    pub fn new() -> Self {
        Self {
            tab_manager: Arc::new(TabManager::new()),
        }
    }
    
    pub fn new_with_tab_manager(tab_manager: Arc<TabManager>) -> Self {
        Self {
            tab_manager,
        }
    }
    
    pub fn get_tab_manager(&self) -> Arc<TabManager> {
        self.tab_manager.clone()
    }
}

impl TabProvider for ChromeTabProvider {
    fn get_tabs(&self) -> Vec<TabItem> {
        self.tab_manager
            .get_tabs()
            .into_iter()
            .map(TabItem::new)
            .collect()
    }
    
    fn search_tabs(&self, query: &str) -> Vec<TabItem> {
        self.tab_manager
            .search_tabs(query)
            .into_iter()
            .map(TabItem::new)
            .collect()
    }
    
    fn update_tabs(&self, tabs: Vec<ChromeTab>) {
        self.tab_manager.update_tabs(tabs);
    }
}