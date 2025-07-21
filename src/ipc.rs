use serde::{Deserialize, Serialize};
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient, ServerOptions, NamedPipeServer};
use std::time::Duration;

pub const PIPE_NAME: &str = r"\\.\pipe\my_launcher_ipc";

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessage {
    GetTabs,
    SwitchToTab { tab_id: i32, window_id: i32 },
    TabList { tabs: Vec<TabInfo> },
    TabSwitchResult { success: bool, error: Option<String> },
    // Command to be sent to Chrome extension
    ChromeCommand { command: ChromeExtensionCommand },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChromeExtensionCommand {
    SwitchToTab { tab_id: i32, window_id: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub id: i32,
    pub window_id: i32,
    pub title: String,
    pub url: String,
    pub fav_icon_url: String,
    pub active: bool,
    pub index: i32,
}

pub async fn send_message<S>(stream: &mut S, message: &IpcMessage) -> io::Result<()>
where
    S: AsyncWriteExt + Unpin,
{
    let json = serde_json::to_string(message)?;
    let json_bytes = json.as_bytes();
    let length = json_bytes.len() as u32;
    
    // Write message length (4 bytes)
    stream.write_all(&length.to_le_bytes()).await?;
    
    // Write message body
    stream.write_all(json_bytes).await?;
    stream.flush().await?;
    
    Ok(())
}

pub async fn read_message<S>(stream: &mut S) -> io::Result<IpcMessage>
where
    S: AsyncReadExt + Unpin,
{
    // Read message length (4 bytes)
    let mut length_bytes = [0u8; 4];
    stream.read_exact(&mut length_bytes).await?;
    let message_length = u32::from_le_bytes(length_bytes) as usize;
    
    // Read message body
    let mut message_bytes = vec![0u8; message_length];
    stream.read_exact(&mut message_bytes).await?;
    
    // Parse JSON
    serde_json::from_slice(&message_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(windows)]
pub async fn create_ipc_server() -> io::Result<NamedPipeServer> {
    ServerOptions::new()
        .first_pipe_instance(true)
        .create(PIPE_NAME)
}

#[cfg(windows)]
pub async fn connect_to_ipc_server() -> io::Result<NamedPipeClient> {
    // Try to connect with retries
    for _ in 0..10 {
        match ClientOptions::new().open(PIPE_NAME) {
            Ok(client) => return Ok(client),
            Err(_) => {
                // Wait a bit before retrying
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
    
    Err(io::Error::new(
        io::ErrorKind::ConnectionRefused,
        "Failed to connect to IPC server after retries",
    ))
}

#[cfg(not(windows))]
pub async fn create_ipc_server() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "IPC server is only supported on Windows",
    ))
}

#[cfg(not(windows))]
pub async fn connect_to_ipc_server() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "IPC client is only supported on Windows",
    ))
}