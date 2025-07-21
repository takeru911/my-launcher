use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeTab {
    pub id: i32,
    pub window_id: i32,
    pub title: String,
    pub url: String,
    pub fav_icon_url: String,
    pub active: bool,
    pub index: i32,
}

#[derive(Debug)]
pub struct TabManager {
    tabs: Arc<Mutex<Vec<ChromeTab>>>,
    command_queue: Arc<Mutex<VecDeque<ChromeCommand>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChromeCommand {
    SwitchToTab { tab_id: i32, window_id: i32 },
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Arc::new(Mutex::new(Vec::new())),
            command_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    
    pub fn update_tabs(&self, tabs: Vec<ChromeTab>) {
        let mut tab_list = self.tabs.lock().unwrap();
        *tab_list = tabs;
    }
    
    pub fn get_tabs(&self) -> Vec<ChromeTab> {
        self.tabs.lock().unwrap().clone()
    }
    
    pub fn search_tabs(&self, query: &str) -> Vec<ChromeTab> {
        let tabs = self.tabs.lock().unwrap();
        if query.is_empty() {
            return tabs.clone();
        }
        
        let query_lower = query.to_lowercase();
        tabs.iter()
            .filter(|tab| {
                tab.title.to_lowercase().contains(&query_lower)
                    || tab.url.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }
    
    pub fn queue_command(&self, command: ChromeCommand) {
        let mut queue = self.command_queue.lock().unwrap();
        queue.push_back(command);
    }
    
    pub fn pop_command(&self) -> Option<ChromeCommand> {
        let mut queue = self.command_queue.lock().unwrap();
        queue.pop_front()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NativeMessage {
    #[serde(rename = "tabList")]
    TabList { tabs: Vec<ChromeTab> },
    #[serde(rename = "switchResult")]
    SwitchResult { 
        success: bool, 
        #[serde(rename = "tabId")]
        tab_id: Option<i32>, 
        error: Option<String> 
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum NativeCommand {
    #[serde(rename = "getTabs")]
    GetTabs,
    #[serde(rename = "switchToTab")]
    SwitchToTab { 
        #[serde(rename = "tabId")]
        tab_id: i32, 
        #[serde(rename = "windowId")]
        window_id: i32 
    },
}