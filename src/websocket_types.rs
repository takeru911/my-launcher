use serde::{Deserialize, Serialize};
use crate::core::native_messaging::ChromeTab;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "request")]
    Request {
        id: String,
        method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<serde_json::Value>,
    },
    #[serde(rename = "response")]
    Response {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<ResponseResult>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<ErrorInfo>,
    },
    #[serde(rename = "event")]
    Event {
        event: EventType,
        data: EventData,
    },
}

// RequestMethodは不要になったので削除

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseResult {
    Tabs { tabs: Vec<ChromeTab> },
    Success { success: bool },
    Pong { timestamp: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventType {
    TabSwitchRequested,
    TabsUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventData {
    TabSwitch { tab_id: i32, window_id: i32 },
    TabsUpdate { tabs: Vec<ChromeTab> },
}

impl WebSocketMessage {
    pub fn request(id: String, method: String, params: Option<serde_json::Value>) -> Self {
        WebSocketMessage::Request { id, method, params }
    }
    
    pub fn response_ok(id: String, result: ResponseResult) -> Self {
        WebSocketMessage::Response {
            id,
            result: Some(result),
            error: None,
        }
    }
    
    pub fn response_error(id: String, code: i32, message: String) -> Self {
        WebSocketMessage::Response {
            id,
            result: None,
            error: Some(ErrorInfo { code, message }),
        }
    }
    
    pub fn event(event: EventType, data: EventData) -> Self {
        WebSocketMessage::Event { event, data }
    }
}