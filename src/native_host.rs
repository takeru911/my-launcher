use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

#[cfg(windows)]
use my_launcher::ipc::{self, IpcMessage, TabInfo as IpcTabInfo};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "command")]
enum Command {
    #[serde(rename = "getTabs")]
    GetTabs,
    #[serde(rename = "switchToTab")]
    SwitchToTab { tabId: i32, windowId: i32 },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Response {
    #[serde(rename = "tabList")]
    TabList { tabs: Vec<TabInfo> },
    #[serde(rename = "switchResult")]
    SwitchResult { success: bool, tabId: Option<i32>, error: Option<String> },
}

#[derive(Debug, Serialize, Deserialize)]
struct TabInfo {
    id: i32,
    #[serde(rename = "windowId")]
    window_id: i32,
    title: String,
    url: String,
    #[serde(rename = "favIconUrl")]
    fav_icon_url: String,
    active: bool,
    index: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum TabListResponse {
    #[serde(rename = "tabList")]
    TabList { tabs: Vec<TabInfo> },
}

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    
    let stdin = io::stdin();
    let stdout = io::stdout();
    
    loop {
        // Read message length (4 bytes)
        let mut length_bytes = [0u8; 4];
        match stdin.lock().read_exact(&mut length_bytes) {
            Ok(_) => {},
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // Chrome closed the connection
                break;
            }
            Err(e) => {
                eprintln!("Error reading message length: {}", e);
                break;
            }
        }
        
        let message_length = u32::from_le_bytes(length_bytes) as usize;
        
        // Validate message length (Chrome Native Messaging has a 1MB limit)
        if message_length == 0 || message_length > 1024 * 1024 {
            eprintln!("Invalid message length: {}", message_length);
            break;
        }
        
        // Read message body
        let mut message_bytes = vec![0u8; message_length];
        if let Err(e) = stdin.lock().read_exact(&mut message_bytes) {
            eprintln!("Error reading message body: {}", e);
            break;
        }
        
        // Parse JSON - could be either a Command or a Response (from Chrome)
        let json_value: serde_json::Value = match serde_json::from_slice(&message_bytes) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error parsing JSON: {}", e);
                continue;
            }
        };
        
        // Check if it's a command or a response
        if let Ok(command) = serde_json::from_value::<Command>(json_value.clone()) {
            let response = handle_command(command).await;
            
            // Send response back
            if let Err(e) = send_response(&mut stdout.lock(), &response) {
                eprintln!("Error sending response: {}", e);
                break;
            }
        } else if let Ok(response) = serde_json::from_value::<TabListResponse>(json_value.clone()) {
            // Handle tab list from Chrome
            handle_chrome_response(response).await;
        } else if json_value.get("type").and_then(|v| v.as_str()) == Some("tabSwitchAck") {
            // Handle tab switch acknowledgment from Chrome
            log::info!("=== NATIVE HOST: Tab switch acknowledgment received ===");
            if let Ok(success) = json_value.get("success").and_then(|v| v.as_bool()).ok_or(()) {
                let tab_id = json_value.get("tabId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let window_id = json_value.get("windowId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let tab_title = json_value.get("tabTitle").and_then(|v| v.as_str()).unwrap_or("Unknown");
                
                log::info!("Tab switch result: success={}, tab_id={}, window_id={}, title={}", 
                          success, tab_id, window_id, tab_title);
                
                // Send acknowledgment to launcher
                #[cfg(windows)]
                {
                    if let Err(e) = communicate_with_launcher(IpcMessage::TabSwitchResult {
                        success,
                        error: if success { None } else { Some("Tab switch failed".to_string()) },
                    }).await {
                        log::error!("Failed to send acknowledgment to launcher: {}", e);
                    }
                }
            }
        } else {
            eprintln!("Unknown message format");
        }
    }
    
    Ok(())
}

fn send_response<W: Write>(writer: &mut W, response: &Response) -> io::Result<()> {
    let json = serde_json::to_string(response)?;
    let json_bytes = json.as_bytes();
    let length = json_bytes.len() as u32;
    
    // Write message length (4 bytes)
    writer.write_all(&length.to_le_bytes())?;
    
    // Write message body
    writer.write_all(json_bytes)?;
    writer.flush()?;
    
    Ok(())
}

async fn handle_command(command: Command) -> Response {
    match command {
        Command::GetTabs => {
            log::info!("=== NATIVE HOST: GetTabs command received ===");
            #[cfg(windows)]
            {
                // Try to communicate with the main launcher
                match communicate_with_launcher(IpcMessage::GetTabs).await {
                    Ok(IpcMessage::TabList { tabs }) => {
                        // Convert IPC tabs to native host tabs
                        let native_tabs = tabs.into_iter().map(|tab| TabInfo {
                            id: tab.id,
                            window_id: tab.window_id,
                            title: tab.title,
                            url: tab.url,
                            fav_icon_url: tab.fav_icon_url,
                            active: tab.active,
                            index: tab.index,
                        }).collect();
                        
                        Response::TabList {
                            tabs: native_tabs,
                        }
                    }
                    Ok(IpcMessage::ChromeCommand { command }) => {
                        use ipc::ChromeExtensionCommand;
                        
                        // Convert the command to a switchToTab response
                        match command {
                            ChromeExtensionCommand::SwitchToTab { tab_id, window_id } => {
                                log::info!("=== NATIVE HOST: CHROME COMMAND RECEIVED ===");
                                log::info!("Command: SwitchToTab");
                                log::info!("Tab ID: {}, Window ID: {}", tab_id, window_id);
                                log::info!("Sending command to Chrome extension via error field");
                                
                                // Return a special response that the Chrome extension will interpret as a command
                                Response::SwitchResult {
                                    success: true,
                                    tabId: Some(tab_id),
                                    error: Some(format!("SWITCH_TAB:{}:{}", tab_id, window_id)),
                                }
                            }
                        }
                    }
                    _ => {
                        eprintln!("Failed to get tabs from launcher");
                        Response::TabList {
                            tabs: vec![],
                        }
                    }
                }
            }
            #[cfg(not(windows))]
            {
                Response::TabList {
                    tabs: vec![],
                }
            }
        }
        Command::SwitchToTab { tabId, windowId } => {
            #[cfg(windows)]
            {
                // Try to communicate with the main launcher
                match communicate_with_launcher(IpcMessage::SwitchToTab { 
                    tab_id: tabId, 
                    window_id: windowId 
                }).await {
                    Ok(IpcMessage::TabSwitchResult { success, error }) => {
                        Response::SwitchResult {
                            success,
                            tabId: if success { Some(tabId) } else { None },
                            error,
                        }
                    }
                    _ => {
                        Response::SwitchResult {
                            success: false,
                            tabId: None,
                            error: Some("Failed to communicate with launcher".to_string()),
                        }
                    }
                }
            }
            #[cfg(not(windows))]
            {
                Response::SwitchResult {
                    success: false,
                    tabId: None,
                    error: Some("Tab switching not supported on this platform".to_string()),
                }
            }
        }
    }
}

#[cfg(windows)]
async fn communicate_with_launcher(message: IpcMessage) -> Result<IpcMessage, io::Error> {
    use log::{error, info};
    
    // Try to connect to the main launcher
    info!("Attempting to connect to launcher IPC server");
    
    match ipc::connect_to_ipc_server().await {
        Ok(mut client) => {
            info!("Connected to launcher IPC server");
            
            // Send the message
            ipc::send_message(&mut client, &message).await?;
            
            // Flush to ensure message is sent
            use tokio::io::AsyncWriteExt;
            client.flush().await?;
            
            // Read the response
            match ipc::read_message(&mut client).await {
                Ok(response) => {
                    info!("Received response from launcher: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    error!("Failed to read response from launcher: {}", e);
                    Err(e)
                }
            }
        }
        Err(e) => {
            error!("Failed to connect to launcher IPC server: {}", e);
            Err(e)
        }
    }
}

async fn handle_chrome_response(response: TabListResponse) {
    match response {
        TabListResponse::TabList { tabs } => {
            log::info!("Received tab list from Chrome with {} tabs", tabs.len());
            
            #[cfg(windows)]
            {
                // Convert TabInfo to ChromeTab and send to launcher
                use my_launcher::core::ChromeTab;
                
                let chrome_tabs: Vec<ChromeTab> = tabs.into_iter().map(|tab| ChromeTab {
                    id: tab.id,
                    window_id: tab.window_id,
                    title: tab.title,
                    url: tab.url,
                    fav_icon_url: tab.fav_icon_url,
                    active: tab.active,
                    index: tab.index,
                }).collect();
                
                // Send the tabs to the launcher via IPC
                if let Err(e) = communicate_with_launcher(IpcMessage::TabList { 
                    tabs: chrome_tabs.into_iter().map(|tab| IpcTabInfo {
                        id: tab.id,
                        window_id: tab.window_id,
                        title: tab.title,
                        url: tab.url,
                        fav_icon_url: tab.fav_icon_url,
                        active: tab.active,
                        index: tab.index,
                    }).collect()
                }).await {
                    log::error!("Failed to send tab list to launcher: {}", e);
                }
            }
        }
    }
}