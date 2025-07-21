use std::sync::Arc;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use log::{info, error, debug, warn};
use serde::Deserialize;

use crate::core::native_messaging::{TabManager, ChromeCommand, ChromeTab};
use crate::websocket_types::{
    WebSocketMessage, ResponseResult, EventType, EventData
};

pub struct WebSocketServer {
    tab_manager: Arc<TabManager>,
    port: u16,
}

impl WebSocketServer {
    pub fn new(tab_manager: Arc<TabManager>, port: u16) -> Self {
        Self { tab_manager, port }
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        let addr = format!("127.0.0.1:{}", self.port);
        info!("Starting WebSocket server on {}", addr);
        
        let listener = TcpListener::bind(&addr).await?;
        info!("WebSocket server listening on {}", addr);
        
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New WebSocket connection from {}", addr);
                    let tab_manager = Arc::clone(&self.tab_manager);
                    tokio::spawn(handle_connection(stream, tab_manager));
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(stream: TcpStream, tab_manager: Arc<TabManager>) {
    let addr = stream.peer_addr().ok();
    info!("Handling WebSocket connection from {:?}", addr);
    
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake failed: {}", e);
            return;
        }
    };
    
    info!("WebSocket handshake successful");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Send initial tab list
    if let Ok(initial_msg) = serde_json::to_string(&WebSocketMessage::event(
        EventType::TabsUpdated,
        EventData::TabsUpdate {
            tabs: tab_manager.get_tabs(),
        },
    )) {
        let _ = ws_sender.send(Message::Text(initial_msg)).await;
    }
    
    // Create a channel for sending tab switch commands
    let (command_tx, mut command_rx) = tokio::sync::mpsc::channel::<(i32, i32)>(32);
    
    // Spawn a task to check for pending commands
    let tab_manager_clone = Arc::clone(&tab_manager);
    let command_tx_clone = command_tx.clone();
    tokio::spawn(async move {
        loop {
            // Check for pending commands every 50ms
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            
            if let Some(command) = tab_manager_clone.pop_command() {
                match command {
                    ChromeCommand::SwitchToTab { tab_id, window_id } => {
                        info!("WebSocket: Found pending command - SwitchToTab({}, {})", tab_id, window_id);
                        let _ = command_tx_clone.send((tab_id, window_id)).await;
                    }
                }
            }
        }
    });
    
    loop {
        tokio::select! {
            // Handle incoming messages from Chrome
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        debug!("Received WebSocket message: {}", text);
                        
                        match serde_json::from_str::<WebSocketMessage>(&text) {
                            Ok(WebSocketMessage::Request { id, method, params }) => {
                                let response = handle_request(id, &method, params, &tab_manager).await;
                                
                                if let Ok(response_text) = serde_json::to_string(&response) {
                                    if let Err(e) = ws_sender.send(Message::Text(response_text)).await {
                                        error!("Failed to send response: {}", e);
                                        break;
                                    }
                                }
                            }
                            Ok(msg) => {
                                debug!("Received non-request message: {:?}", msg);
                            }
                            Err(e) => {
                                error!("Failed to parse WebSocket message: {}", e);
                            }
                        }
                    }
                    Some(Ok(Message::Binary(_))) => {
                        warn!("Received binary message, ignoring");
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket connection closed by client");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                            error!("Failed to send pong: {}", e);
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        debug!("Received pong");
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended");
                        break;
                    }
                    _ => {}
                }
            }
            
            // Handle pending tab switch commands
            Some((tab_id, window_id)) = command_rx.recv() => {
                info!("Sending tab switch event to Chrome: tab_id={}, window_id={}", tab_id, window_id);
                
                let event = WebSocketMessage::event(
                    EventType::TabSwitchRequested,
                    EventData::TabSwitch { tab_id, window_id },
                );
                
                if let Ok(event_text) = serde_json::to_string(&event) {
                    if let Err(e) = ws_sender.send(Message::Text(event_text)).await {
                        error!("Failed to send tab switch event: {}", e);
                        break;
                    }
                }
            }
        }
    }
    
    info!("WebSocket connection closed");
}

async fn handle_request(
    id: String,
    method: &str,
    params: Option<serde_json::Value>,
    tab_manager: &Arc<TabManager>,
) -> WebSocketMessage {
    match method {
        "getTabs" => {
            let tabs = tab_manager.get_tabs();
            info!("WebSocket: GetTabs request, returning {} tabs", tabs.len());
            WebSocketMessage::response_ok(id, ResponseResult::Tabs { tabs })
        }
        
        "updateTabs" => {
            if let Some(params) = params {
                if let Ok(tabs_data) = serde_json::from_value::<UpdateTabsParams>(params) {
                    info!("WebSocket: UpdateTabs request, updating {} tabs", tabs_data.tabs.len());
                    tab_manager.update_tabs(tabs_data.tabs);
                    WebSocketMessage::response_ok(id, ResponseResult::Success { success: true })
                } else {
                    WebSocketMessage::response_error(id, 400, "Invalid updateTabs params".to_string())
                }
            } else {
                WebSocketMessage::response_error(id, 400, "Missing params for updateTabs".to_string())
            }
        }
        
        "switchToTab" => {
            if let Some(params) = params {
                if let Ok(switch_params) = serde_json::from_value::<SwitchTabParams>(params) {
                    info!("WebSocket: SwitchToTab request, tab_id={}, window_id={}", switch_params.tab_id, switch_params.window_id);
                    
                    // Queue the command for the main app
                    tab_manager.queue_command(ChromeCommand::SwitchToTab { 
                        tab_id: switch_params.tab_id, 
                        window_id: switch_params.window_id 
                    });
                    
                    WebSocketMessage::response_ok(id, ResponseResult::Success { success: true })
                } else {
                    WebSocketMessage::response_error(id, 400, "Invalid switchToTab params".to_string())
                }
            } else {
                WebSocketMessage::response_error(id, 400, "Missing params for switchToTab".to_string())
            }
        }
        
        "keepAlive" => {
            debug!("WebSocket: KeepAlive request");
            let timestamp = chrono::Utc::now().timestamp_millis();
            WebSocketMessage::response_ok(id, ResponseResult::Pong { timestamp })
        }
        
        _ => {
            error!("Unknown method: {}", method);
            WebSocketMessage::response_error(id, 404, format!("Unknown method: {}", method))
        }
    }
}

#[derive(Debug, Deserialize)]
struct UpdateTabsParams {
    tabs: Vec<ChromeTab>,
}

#[derive(Debug, Deserialize)]
struct SwitchTabParams {
    tab_id: i32,
    window_id: i32,
}