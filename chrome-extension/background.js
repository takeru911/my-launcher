// WebSocket client for My Launcher Chrome Extension
class WebSocketClient {
    constructor() {
        this.ws = null;
        this.reconnectDelay = 1000;
        this.maxReconnectDelay = 30000;
        this.keepAliveInterval = null;
        this.requestId = 0;
        this.pendingRequests = new Map();
        this.isConnected = false;
    }
    
    connect() {
        console.log('WebSocket: Attempting to connect to ws://localhost:9999');
        
        try {
            this.ws = new WebSocket('ws://localhost:9999');
            
            this.ws.onopen = () => {
                console.log('=== WEBSOCKET CONNECTED ===');
                this.isConnected = true;
                this.reconnectDelay = 1000; // Reset reconnect delay
                this.startKeepAlive();
                this.sendTabUpdate();
                
                // Store connection state
                chrome.storage.local.set({ wsConnected: true });
            };
            
            this.ws.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    console.log('WebSocket message received:', message);
                    this.handleMessage(message);
                } catch (e) {
                    console.error('Failed to parse WebSocket message:', e);
                }
            };
            
            this.ws.onclose = (event) => {
                console.log('=== WEBSOCKET DISCONNECTED ===', event.code, event.reason);
                this.isConnected = false;
                this.stopKeepAlive();
                this.clearPendingRequests();
                chrome.storage.local.set({ wsConnected: false });
                this.scheduleReconnect();
            };
            
            this.ws.onerror = (error) => {
                console.error('WebSocket error:', error);
            };
        } catch (e) {
            console.error('Failed to create WebSocket:', e);
            this.scheduleReconnect();
        }
    }
    
    scheduleReconnect() {
        console.log(`Scheduling reconnect in ${this.reconnectDelay}ms`);
        setTimeout(() => this.connect(), this.reconnectDelay);
        this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
    }
    
    startKeepAlive() {
        this.stopKeepAlive();
        this.keepAliveInterval = setInterval(() => {
            if (this.isConnected) {
                this.sendRequest('keepAlive', {});
            }
        }, 30000); // Send keep-alive every 30 seconds
    }
    
    stopKeepAlive() {
        if (this.keepAliveInterval) {
            clearInterval(this.keepAliveInterval);
            this.keepAliveInterval = null;
        }
    }
    
    clearPendingRequests() {
        for (const [id, request] of this.pendingRequests) {
            if (request.reject) {
                request.reject(new Error('WebSocket disconnected'));
            }
        }
        this.pendingRequests.clear();
    }
    
    generateRequestId() {
        return `req_${++this.requestId}_${Date.now()}`;
    }
    
    sendRequest(method, params = {}) {
        return new Promise((resolve, reject) => {
            if (!this.isConnected) {
                reject(new Error('WebSocket not connected'));
                return;
            }
            
            const id = this.generateRequestId();
            const message = {
                type: 'request',
                id,
                method,
                params
            };
            
            this.pendingRequests.set(id, { resolve, reject });
            
            try {
                this.ws.send(JSON.stringify(message));
                console.log('Sent request:', message);
            } catch (e) {
                this.pendingRequests.delete(id);
                reject(e);
            }
        });
    }
    
    sendMessage(message) {
        if (!this.isConnected) {
            console.warn('Cannot send message: WebSocket not connected');
            return;
        }
        
        try {
            this.ws.send(JSON.stringify(message));
        } catch (e) {
            console.error('Failed to send message:', e);
        }
    }
    
    handleMessage(message) {
        // Handle response messages
        if (message.type === 'response') {
            const pending = this.pendingRequests.get(message.id);
            if (pending) {
                this.pendingRequests.delete(message.id);
                if (message.error) {
                    pending.reject(new Error(message.error.message));
                } else {
                    pending.resolve(message.result);
                }
            }
            return;
        }
        
        // Handle event messages
        if (message.type === 'event') {
            switch (message.event) {
                case 'tabSwitchRequested':
                    console.log('=== TAB SWITCH EVENT RECEIVED ===');
                    console.log('Tab ID:', message.data.tab_id);
                    console.log('Window ID:', message.data.window_id);
                    this.executeSwitchToTab(message.data.tab_id, message.data.window_id);
                    break;
                    
                case 'tabsUpdated':
                    console.log('Tabs updated event received');
                    // Optionally handle tabs update from server
                    break;
                    
                default:
                    console.warn('Unknown event type:', message.event);
            }
        }
    }
    
    async sendTabUpdate() {
        console.log('Sending tab update to server');
        
        try {
            const tabs = await chrome.tabs.query({});
            const tabData = tabs.map(tab => ({
                id: tab.id,
                window_id: tab.windowId,
                title: tab.title || tab.url || 'Untitled',
                url: tab.url || '',
                fav_icon_url: tab.favIconUrl || '',
                active: tab.active,
                index: tab.index
            }));
            
            console.log(`Sending ${tabData.length} tabs to server`);
            await this.sendRequest('updateTabs', { tabs: tabData });
        } catch (e) {
            console.error('Failed to send tab update:', e);
        }
    }
    
    executeSwitchToTab(tabId, windowId) {
        console.log('=== EXECUTING TAB SWITCH ===');
        console.log(`Tab ID: ${tabId}, Window ID: ${windowId}`);
        
        if (!tabId || !windowId) {
            console.error('Invalid tab or window ID:', { tabId, windowId });
            return;
        }
        
        // First verify the tab exists
        chrome.tabs.get(tabId, (tab) => {
            if (chrome.runtime.lastError) {
                console.error('Tab not found:', chrome.runtime.lastError);
                return;
            }
            
            console.log('Tab found:', tab.title);
            
            // Focus the window first
            chrome.windows.update(windowId, { focused: true }, (window) => {
                if (chrome.runtime.lastError) {
                    console.error('Failed to focus window:', chrome.runtime.lastError);
                    return;
                }
                
                console.log('Window focused successfully');
                
                // Then activate the tab
                chrome.tabs.update(tabId, { active: true }, (tab) => {
                    if (chrome.runtime.lastError) {
                        console.error('Failed to activate tab:', chrome.runtime.lastError);
                    } else {
                        console.log('=== TAB SWITCH SUCCESSFUL ===');
                        console.log('Switched to:', tab.title);
                        
                        // Send success acknowledgment
                        this.sendRequest('switchToTab', { 
                            tab_id: tabId, 
                            window_id: windowId 
                        }).catch(e => {
                            console.error('Failed to send switch acknowledgment:', e);
                        });
                    }
                });
            });
        });
    }
}

// Create global WebSocket client instance
let wsClient = new WebSocketClient();

// Native Messaging support has been removed in favor of WebSocket

// Tab change listeners
chrome.tabs.onCreated.addListener((tab) => {
    console.log('Tab created:', tab.id);
    wsClient.sendTabUpdate();
});

chrome.tabs.onRemoved.addListener((tabId) => {
    console.log('Tab removed:', tabId);
    wsClient.sendTabUpdate();
});

chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
    if (changeInfo.status === 'complete' || changeInfo.title || changeInfo.url) {
        console.log('Tab updated:', tabId);
        wsClient.sendTabUpdate();
    }
});

chrome.tabs.onActivated.addListener((activeInfo) => {
    console.log('Tab activated:', activeInfo.tabId);
    wsClient.sendTabUpdate();
});

chrome.tabs.onMoved.addListener((tabId, moveInfo) => {
    console.log('Tab moved:', tabId);
    wsClient.sendTabUpdate();
});

// Initialize on extension load
chrome.runtime.onInstalled.addListener(() => {
    console.log('Extension installed/updated');
    wsClient.connect();
});

chrome.runtime.onStartup.addListener(() => {
    console.log('Extension startup');
    wsClient.connect();
});

// Connect immediately
wsClient.connect();

// WebSocket reconnection is handled automatically by the WebSocketClient class

// Keep service worker alive
chrome.alarms.create('keepAlive', { periodInMinutes: 0.5 });
chrome.alarms.onAlarm.addListener((alarm) => {
    if (alarm.name === 'keepAlive') {
        console.log('Keep-alive alarm triggered');
        // Just being active keeps the service worker alive
    }
});

console.log('My Launcher Chrome Extension with WebSocket support initialized');