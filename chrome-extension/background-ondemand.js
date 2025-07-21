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
    // Reconnect after a delay
    setTimeout(connect, 5000);
  });
}

// Handle messages from native host
function handleNativeMessage(message) {
  console.log('Handling message:', message);
  
  // Check for special switch tab command in error field
  if (message.type === 'switchResult' && message.error && message.error.startsWith('SWITCH_TAB:')) {
    // Parse the command from the error field
    const parts = message.error.split(':');
    if (parts.length === 3) {
      const tabId = parseInt(parts[1]);
      const windowId = parseInt(parts[2]);
      console.log('Executing tab switch from command:', tabId, windowId);
      executeSwitchToTab(tabId, windowId);
    }
    return;
  }
  
  // Handle getTabs command - only send tabs when requested
  if (message.command === 'getTabs' || (message.type === 'tabList' && message.tabs && message.tabs.length === 0)) {
    console.log('Tab list requested, fetching all tabs...');
    // Get all tabs across all windows
    chrome.tabs.query({}, (tabs) => {
      console.log('Found', tabs.length, 'tabs across all windows');
      
      // Group tabs by window for debugging
      const tabsByWindow = tabs.reduce((acc, tab) => {
        if (!acc[tab.windowId]) acc[tab.windowId] = [];
        acc[tab.windowId].push(tab);
        return acc;
      }, {});
      console.log('Tabs by window:', Object.keys(tabsByWindow).map(wId => `Window ${wId}: ${tabsByWindow[wId].length} tabs`));
      
      const tabData = tabs.map(tab => ({
        id: tab.id,
        windowId: tab.windowId,
        title: tab.title || '',
        url: tab.url || '',
        favIconUrl: tab.favIconUrl || '',
        active: tab.active,
        index: tab.index
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
  if (tabId && windowId) {
    // First, focus the window
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
      
      // Then activate the tab
      chrome.tabs.update(tabId, { active: true }, (tab) => {
        if (chrome.runtime.lastError) {
          console.error('Failed to activate tab:', chrome.runtime.lastError);
          sendMessage({
            type: 'switchResult',
            success: false,
            error: chrome.runtime.lastError.message
          });
        } else {
          console.log('Successfully switched to tab:', tabId);
          sendMessage({
            type: 'switchResult',
            success: true,
            tabId: tabId
          });
        }
      });
    });
  } else {
    sendMessage({
      type: 'switchResult',
      success: false,
      error: 'Invalid tab or window ID'
    });
  }
}

// Send message to native host
function sendMessage(message) {
  if (port) {
    port.postMessage(message);
  } else {
    console.error('No connection to native host');
  }
}

// Initialize connection when extension loads
connect();

// Reconnect if needed
chrome.runtime.onStartup.addListener(() => {
  connect();
});

// Handle extension installation/update
chrome.runtime.onInstalled.addListener(() => {
  console.log('Extension installed/updated');
});

// Manual debug function
function debugGetAllTabs() {
  chrome.tabs.query({}, (tabs) => {
    console.log('=== Debug: All tabs ===');
    console.log('Total tabs:', tabs.length);
    
    // Get all windows
    chrome.windows.getAll({ populate: true }, (windows) => {
      console.log('Total windows:', windows.length);
      windows.forEach(window => {
        console.log(`Window ${window.id}: ${window.tabs.length} tabs`);
      });
    });
    
    // Check for any permission issues
    chrome.permissions.getAll((permissions) => {
      console.log('Extension permissions:', permissions);
    });
  });
}