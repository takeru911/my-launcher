// タブの詳細情報をデバッグ
async function debugAllTabs() {
  console.log('=== Tab Debug Info ===');
  
  // 1. 通常のタブ取得
  chrome.tabs.query({}, (tabs) => {
    console.log('Normal query - Total tabs:', tabs.length);
    
    // タブの状態を詳しく見る
    tabs.forEach(tab => {
      console.log(`Tab ${tab.id}:`, {
        title: tab.title,
        url: tab.url,
        status: tab.status,
        discarded: tab.discarded, // スリープ中かどうか
        active: tab.active,
        windowId: tab.windowId,
        index: tab.index
      });
    });
  });
  
  // 2. スリープ中のタブも含めて取得
  chrome.tabs.query({ discarded: true }, (discardedTabs) => {
    console.log('Discarded tabs:', discardedTabs.length);
  });
  
  // 3. すべてのウィンドウとタブ
  chrome.windows.getAll({ populate: true }, (windows) => {
    console.log('Total windows:', windows.length);
    let totalTabs = 0;
    windows.forEach(window => {
      console.log(`Window ${window.id}: ${window.tabs.length} tabs, focused: ${window.focused}`);
      totalTabs += window.tabs.length;
    });
    console.log('Total tabs across all windows:', totalTabs);
  });
  
  // 4. タブグループの情報
  if (chrome.tabGroups) {
    chrome.tabGroups.query({}, (groups) => {
      console.log('Tab groups:', groups.length);
    });
  }
  
  // 5. 最近閉じたタブ
  if (chrome.sessions) {
    chrome.sessions.getRecentlyClosed({ maxResults: 10 }, (sessions) => {
      const closedTabs = sessions.filter(s => s.tab);
      console.log('Recently closed tabs:', closedTabs.length);
    });
  }
}

// スリープ中のタブも含めてすべて取得
function getAllTabsIncludingDiscarded(callback) {
  chrome.tabs.query({}, (tabs) => {
    console.log('Getting all tabs including discarded ones...');
    
    // タブの詳細情報を含める
    const detailedTabs = tabs.map(tab => ({
      id: tab.id,
      windowId: tab.windowId,
      title: tab.title || tab.url || 'Untitled',
      url: tab.url || '',
      favIconUrl: tab.favIconUrl || '',
      active: tab.active,
      index: tab.index,
      status: tab.status,
      discarded: tab.discarded || false,
      pinned: tab.pinned || false
    }));
    
    callback(detailedTabs);
  });
}

// 実行
debugAllTabs();