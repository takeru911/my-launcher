// Native messaging host name
const NATIVE_HOST = 'com.mylauncher.tabconnector';

// Native messaging port
let port = null;

// Connect to native host
function connect() {
  console.log('Connecting to native host...');
  port = chrome.runtime.connectNative(NATIVE_HOST);
  
  port.onMessage.addListener((message) => {
    console.log('Received from native host:', message);
    handleNativeMessage(message);
  });
  
  port.onDisconnect.addListener(() => {
    console.log('Disconnected from native host');
    if (chrome.runtime.lastError) {
      console.error('Connection error:', chrome.runtime.lastError.message);
    }
    port = null;
  });
}

// Handle messages from native host
function handleNativeMessage(message) {
  console.log('=== CHROME EXTENSION: Message received ===');
  console.log('Message type:', message.type);
  console.log('Full message:', JSON.stringify(message, null, 2));
  
  // Check for special switch tab command in error field
  if (message.type === 'switchResult' && message.error && message.error.startsWith('SWITCH_TAB:')) {
    console.log('=== TAB SWITCH COMMAND DETECTED ===');
    // Parse the command from the error field
    const parts = message.error.split(':');
    if (parts.length === 3) {
      const tabId = parseInt(parts[1]);
      const windowId = parseInt(parts[2]);
      console.log(`Switching to tab: ID=${tabId}, Window=${windowId}`);
      executeSwitchToTab(tabId, windowId);
    } else {
      console.error('Invalid SWITCH_TAB command format:', message.error);
    }
    return;
  }
  
  // Handle response from native host - if empty, send actual tabs
  if (message.type === 'tabList' && message.tabs && message.tabs.length === 0) {
    console.log('Native host returned empty tab list, sending actual tabs...');
    // Get all tabs and send them back
    chrome.tabs.query({}, (tabs) => {
      console.log('Sending', tabs.length, 'tabs to native host');
      const tabData = tabs.map(tab => ({
        id: tab.id,
        windowId: tab.windowId,
        title: tab.title || tab.url || 'Untitled',
        url: tab.url || '',
        favIconUrl: tab.favIconUrl || '',
        active: tab.active,
        index: tab.index,
        status: tab.status,
        discarded: tab.discarded || false
      }));
      
      sendMessage({
        type: 'tabList',
        tabs: tabData
      });
    });
    return;
  }
  
  // If launcher already has tabs, don't send them again
  if (message.type === 'tabList') {
    console.log('Launcher already has', message.tabs.length, 'tabs');
    return;
  }
  
  if (message.command === 'getTabs') {
    // Get all tabs and send back to native host
    chrome.tabs.query({}, (tabs) => {
      const tabData = tabs.map(tab => ({
        id: tab.id,
        windowId: tab.windowId,
        title: tab.title || tab.url || 'Untitled', // タイトルがない場合はURLを使用
        url: tab.url || '',
        favIconUrl: tab.favIconUrl || '',
        active: tab.active,
        index: tab.index,
        // 追加情報
        status: tab.status,
        discarded: tab.discarded || false
      }));
      
      sendMessage({
        type: 'tabList',
        tabs: tabData
      });
    });
  } else if (message.command === 'switchToTab') {
    // Switch to specific tab
    const tabId = message.tabId;
    const windowId = message.windowId;
    executeSwitchToTab(tabId, windowId);
  }
}

// Execute tab switch
function executeSwitchToTab(tabId, windowId) {
  console.log('=== EXECUTING TAB SWITCH ===');
  console.log(`Tab ID: ${tabId}, Window ID: ${windowId}`);
  
  if (!tabId || !windowId) {
    console.error('Invalid tab or window ID:', { tabId, windowId });
    sendMessage({
      type: 'switchResult',
      success: false,
      error: 'Invalid tab or window ID'
    });
    return;
  }
  
  // First verify the tab exists
  chrome.tabs.get(tabId, (tab) => {
    if (chrome.runtime.lastError) {
      console.error('Tab not found:', chrome.runtime.lastError);
      sendMessage({
        type: 'switchResult',
        success: false,
        error: `Tab not found: ${chrome.runtime.lastError.message}`
      });
      return;
    }
    
    console.log('Tab found:', tab.title);
    
    // Focus the window
    chrome.windows.update(windowId, { focused: true }, (window) => {
      if (chrome.runtime.lastError) {
        console.error('Failed to focus window:', chrome.runtime.lastError);
        sendMessage({
          type: 'switchResult',
          success: false,
          error: chrome.runtime.lastError.message
        });
        return;
      }
      
      console.log('Window focused successfully');
      
      // Activate the tab
      chrome.tabs.update(tabId, { active: true }, (tab) => {
        if (chrome.runtime.lastError) {
          console.error('Failed to activate tab:', chrome.runtime.lastError);
          sendMessage({
            type: 'switchResult',
            success: false,
            error: chrome.runtime.lastError.message
          });
        } else {
          console.log('=== TAB SWITCH SUCCESSFUL ===');
          console.log('Switched to:', tab.title);
          
          // Send acknowledgment back to native host
          sendMessage({
            type: 'tabSwitchAck',
            success: true,
            tabId: tabId,
            windowId: windowId,
            tabTitle: tab.title
          });
        }
      });
    });
  });
}

// Send message to native host
function sendMessage(message) {
  if (port) {
    port.postMessage(message);
  } else {
    console.error('No connection to native host');
  }
}

// Polling interval in milliseconds
const POLL_INTERVAL = 500; // Poll every 500ms for faster response

// Start polling for commands
function startPolling() {
  console.log('Starting polling with interval:', POLL_INTERVAL, 'ms');
  setInterval(() => {
    if (port) {
      // Send getTabs command to check for pending commands
      sendMessage({
        command: 'getTabs'
      });
    } else {
      console.warn('No connection to native host, skipping poll');
    }
  }, POLL_INTERVAL);
}

// Initialize connection when extension loads
connect();
startPolling();

// Reconnect if needed
chrome.runtime.onStartup.addListener(() => {
  connect();
});

// Handle extension installation/update
chrome.runtime.onInstalled.addListener(() => {
  console.log('Extension installed/updated');
});

// Also send initial getTabs to populate launcher
setTimeout(() => {
  if (port) {
    sendMessage({
      command: 'getTabs'
    });
  }
}, 1000);