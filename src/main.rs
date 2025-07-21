use eframe::egui;
use my_launcher::core::{
    launcher::LauncherCore,
    search_engine::{SearchMode, SearchResult},
    window_manager::WindowsApiManager,
    BrowserSearchEngine,
    native_messaging::TabManager,
};
use my_launcher::ui::alt_tab_grid::{AltTabGrid, GridItem};
use my_launcher::ui::browser_list::BrowserList;
use my_launcher::window_thumbnail::ThumbnailCache;
use std::sync::Arc;
use std::error::Error;
use std::time::{Duration, Instant};
#[cfg(windows)]
use std::thread;
#[cfg(windows)]
use tokio::runtime::Runtime;

fn setup_custom_fonts(ctx: &egui::Context) -> Result<(), Box<dyn Error>> {
    let mut fonts = egui::FontDefinitions::default();
    
    // Try to load a Windows Japanese font
    let font_paths = vec![
        "C:\\Windows\\Fonts\\msgothic.ttc",
        "C:\\Windows\\Fonts\\YuGothM.ttc", 
        "C:\\Windows\\Fonts\\meiryo.ttc",
        "C:\\Windows\\Fonts\\msmincho.ttc",
    ];
    
    let mut font_loaded = false;
    for font_path in font_paths {
        match std::fs::read(font_path) {
            Ok(font_data) => {
                log::info!("Successfully loaded font from: {}", font_path);
                fonts.font_data.insert(
                    "japanese".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                font_loaded = true;
                break;
            }
            Err(e) => {
                log::debug!("Failed to load font from {}: {}", font_path, e);
            }
        }
    }
    
    if font_loaded {
        // Insert Japanese font at the beginning for both families
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "japanese".to_owned());
            
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "japanese".to_owned());
            
        ctx.set_fonts(fonts);
        Ok(())
    } else {
        Err("No Japanese fonts found".into())
    }
}

// SearchResultをGridItemとして扱うためのラッパー
struct SearchResultItem<'a>(&'a SearchResult);

impl<'a> GridItem for SearchResultItem<'a> {
    fn title(&self) -> &str {
        &self.0.title
    }

    fn description(&self) -> &str {
        &self.0.description
    }

    fn hwnd(&self) -> isize {
        self.0.window_info.as_ref().map(|w| w.hwnd).unwrap_or(0)
    }

    fn id(&self) -> String {
        match &self.0.action {
            my_launcher::core::search_engine::Action::SwitchWindow(hwnd) => hwnd.to_string(),
            my_launcher::core::search_engine::Action::GoogleSearch(query) => format!("google:{}", query),
            my_launcher::core::search_engine::Action::OpenBookmark(url) => format!("bookmark:{}", url),
            my_launcher::core::search_engine::Action::OpenHistory(url) => format!("history:{}", url),
            my_launcher::core::search_engine::Action::SwitchToTab { tab_id, window_id } => format!("tab:{}:{}", tab_id, window_id),
        }
    }
}

struct LauncherApp {
    input_text: String,
    mode: SearchMode,
    core: LauncherCore<BrowserSearchEngine, WindowsApiManager>,
    search_results: Vec<SearchResult>,
    grid: AltTabGrid,
    browser_list: BrowserList,
    thumbnail_cache: ThumbnailCache,
    first_frame: bool,
    last_input_change: Option<Instant>,
    pending_search_text: Option<String>,
    debounce_duration: Duration,
    tab_manager: Arc<TabManager>,
    status_message: Option<String>,
    status_timestamp: Option<Instant>,
}

impl LauncherApp {
    fn new() -> Self {
        let tab_manager = Arc::new(TabManager::new());
        Self::new_with_tab_manager(tab_manager)
    }
    
    fn new_with_tab_manager(tab_manager: Arc<TabManager>) -> Self {
        let window_manager = Arc::new(WindowsApiManager);
        let search_engine = BrowserSearchEngine::new_with_tab_manager(Arc::clone(&tab_manager));
        let mut core = LauncherCore::new(search_engine, window_manager);
        
        // 初期状態でウィンドウ情報を更新
        core.refresh_windows();
        
        let mut app = Self {
            input_text: String::new(),
            mode: SearchMode::Windows, // Windowsモードから開始
            core,
            search_results: Vec::new(),
            grid: AltTabGrid::new(),
            browser_list: BrowserList::new(),
            thumbnail_cache: ThumbnailCache::new(),
            first_frame: true,
            last_input_change: None,
            pending_search_text: None,
            debounce_duration: Duration::from_millis(500), // 500msのデバウンス（調整可能）
            tab_manager,
            status_message: None,
            status_timestamp: None,
        };
        
        // 初期表示のために検索を実行
        app.update_search();
        app
    }

    fn switch_mode(&mut self) {
        self.mode = match self.mode {
            SearchMode::Browser => SearchMode::Windows,
            SearchMode::Windows => SearchMode::Browser,
        };
        self.grid.selected_index = 0;
        self.browser_list.selected_index = 0;
        // モード切り替え時は即座に検索
        self.force_search();
    }

    fn update_search(&mut self) {
        let old_query = self.search_results.first().map(|r| r.title.clone());
        self.search_results = self.core.search(&self.input_text, self.mode);
        let new_query = self.search_results.first().map(|r| r.title.clone());
        
        // 検索クエリが変わった場合、BrowserListをリセット
        if old_query != new_query {
            self.browser_list.reset_for_new_search();
        }
        
        if self.grid.selected_index >= self.search_results.len() && !self.search_results.is_empty() {
            self.grid.selected_index = self.search_results.len() - 1;
        }
        if self.browser_list.selected_index >= self.search_results.len() && !self.search_results.is_empty() {
            self.browser_list.selected_index = self.search_results.len() - 1;
        }
    }
    
    fn force_search(&mut self) {
        // デバウンスをキャンセルして即座に検索
        self.last_input_change = None;
        self.pending_search_text = None;
        self.update_search();
    }

    fn execute_selected(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.search_results.get(self.grid.selected_index) {
            // Special handling for tab switching
            match &result.action {
                my_launcher::core::search_engine::Action::SwitchToTab { tab_id, window_id } => {
                    use my_launcher::core::native_messaging::ChromeCommand;
                    log::info!("=== TAB SWITCH INITIATED ===");
                    log::info!("Tab ID: {}, Window ID: {}", tab_id, window_id);
                    log::info!("Selected result: {} - {}", result.title, result.description);
                    
                    // Set status message
                    self.status_message = Some(format!("Switching to tab: {}", result.title));
                    self.status_timestamp = Some(Instant::now());
                    
                    // Queue the command
                    self.tab_manager.queue_command(ChromeCommand::SwitchToTab {
                        tab_id: *tab_id,
                        window_id: *window_id,
                    });
                    log::info!("Command queued successfully");
                    
                    // Also execute the action to bring Chrome to front
                    log::info!("Bringing Chrome window to front");
                    self.core.execute_action(&result.action);
                }
                _ => {
                    // For other actions, just execute normally
                    self.core.execute_action(&result.action);
                }
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn show_browser_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if !self.search_results.is_empty() {
            // BrowserListコンポーネントを使用
            if let Some(clicked_index) = self.browser_list.render(ui, &self.search_results) {
                self.browser_list.selected_index = clicked_index;
                self.execute_selected(ctx);
            }
            
            // 選択インデックスを同期
            self.grid.selected_index = self.browser_list.selected_index;
        } else if !self.input_text.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Press Enter to search on Google");
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Type something to search");
            });
        }
    }

    fn handle_keyboard_input(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Tab: モード切り替え
        if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
            self.switch_mode();
        }

        match self.mode {
            SearchMode::Windows => {
                // Windowsモード: グリッドナビゲーション
                if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    if self.grid.selected_index < self.search_results.len().saturating_sub(1) {
                        self.grid.selected_index += 1;
                    }
                }

                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    if self.grid.selected_index > 0 {
                        self.grid.selected_index -= 1;
                    }
                }

                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    let new_index = self.grid.selected_index + self.grid.columns;
                    if new_index < self.search_results.len() {
                        self.grid.selected_index = new_index;
                    }
                }

                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    if self.grid.selected_index >= self.grid.columns {
                        self.grid.selected_index -= self.grid.columns;
                    }
                }
            }
            SearchMode::Browser => {
                // Browserモード: BrowserListのキーボード処理を使用
                self.browser_list.handle_keyboard(ui, self.search_results.len());
                // 選択インデックスを同期
                self.grid.selected_index = self.browser_list.selected_index;
            }
        }

        // 共通のキーボードショートカット
        // Home/End: 最初/最後の項目へ
        if ui.input(|i| i.key_pressed(egui::Key::Home)) {
            self.grid.selected_index = 0;
        }

        if ui.input(|i| i.key_pressed(egui::Key::End)) && !self.search_results.is_empty() {
            self.grid.selected_index = self.search_results.len() - 1;
        }

        // Enter: 選択項目を実行
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if self.search_results.get(self.grid.selected_index).is_some() {
                self.execute_selected(ctx);
            }
        }

        // Esc: 終了
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // デバウンス処理：一定時間経過後に検索を実行
        if let (Some(last_change), Some(pending_text)) = (self.last_input_change, &self.pending_search_text) {
            if last_change.elapsed() >= self.debounce_duration {
                // デバウンス時間が経過したら検索を実行
                if &self.input_text == pending_text {
                    self.update_search();
                    self.last_input_change = None;
                    self.pending_search_text = None;
                } else {
                    // 入力が更に変更されている場合は、再度待機
                    self.last_input_change = Some(Instant::now());
                    self.pending_search_text = Some(self.input_text.clone());
                }
                // 再描画をリクエスト
                ctx.request_repaint();
            } else {
                // まだデバウンス時間が経過していない場合は、定期的に再描画をリクエスト
                ctx.request_repaint_after(Duration::from_millis(50));
            }
        }
        
        // 初回実行時に日本語フォントを設定
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            if let Err(e) = setup_custom_fonts(ctx) {
                log::warn!("Failed to load Japanese fonts: {}", e);
            }
        });

        // ダークテーマを適用
        ctx.set_visuals(egui::Visuals::dark());
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // 検索バー
            ui.horizontal(|ui| {
                ui.label(format!("Mode: {:?}", self.mode));
                
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.input_text)
                        .desired_width(400.0)
                        .hint_text(match self.mode {
                            SearchMode::Browser => "Search web, bookmarks, history...",
                            SearchMode::Windows => "Search windows...",
                        })
                        .id(egui::Id::new("search_input"))
                );

                // 初回フレームでフォーカスを設定
                if self.first_frame {
                    response.request_focus();
                    self.first_frame = false;
                }

                if response.changed() {
                    self.grid.selected_index = 0;
                    
                    match self.mode {
                        SearchMode::Windows => {
                            // Windowsモードでは即座に検索
                            self.update_search();
                        }
                        SearchMode::Browser => {
                            // Browserモードではデバウンス処理
                            self.last_input_change = Some(Instant::now());
                            self.pending_search_text = Some(self.input_text.clone());
                        }
                    }
                }

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    // Enter キーが押された場合は即座に検索して実行
                    self.force_search();
                    self.execute_selected(ctx);
                }
            });

            // Display status message if present
            if let Some(status) = &self.status_message {
                if let Some(timestamp) = self.status_timestamp {
                    let elapsed = timestamp.elapsed();
                    // Show status for 3 seconds
                    if elapsed < Duration::from_secs(3) {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 10.0;
                            ui.add(egui::Spinner::new());
                            ui.label(egui::RichText::new(status).color(egui::Color32::from_rgb(100, 200, 255)));
                        });
                    } else {
                        // Clear status after timeout
                        self.status_message = None;
                        self.status_timestamp = None;
                    }
                }
            }

            ui.separator();

            // モードに応じてUIを切り替え
            match self.mode {
                SearchMode::Windows => {
                    // Windowsモード: Alt+Tabスタイルのグリッド表示
                    if !self.search_results.is_empty() {
                        let items: Vec<SearchResultItem> = self.search_results.iter()
                            .map(|r| SearchResultItem(r))
                            .collect();

                        if let Some(clicked_index) = self.grid.show(ui, ctx, &items, &mut self.thumbnail_cache) {
                            self.grid.selected_index = clicked_index;
                            self.execute_selected(ctx);
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("No windows found");
                        });
                    }
                }
                SearchMode::Browser => {
                    // Browserモード: シンプルなリスト表示
                    self.show_browser_ui(ui, ctx);
                }
            }

            // キーボードショートカット処理
            self.handle_keyboard_input(ui, ctx);
        });
    }
}

#[cfg(windows)]
async fn run_ipc_server(tab_manager: Arc<TabManager>) {
    use my_launcher::ipc::{self, IpcMessage, TabInfo};
    use log::{info, error};
    
    info!("Starting IPC server");
    
    loop {
        match ipc::create_ipc_server().await {
            Ok(mut server) => {
                info!("IPC server listening on {}", ipc::PIPE_NAME);
                
                // Accept a connection
                match server.connect().await {
                    Ok(_) => {
                        info!("Client connected to IPC server");
                        
                        // Handle messages
                        loop {
                            match ipc::read_message(&mut server).await {
                                Ok(message) => {
                                    info!("Received IPC message: {:?}", message);
                                    
                                    let response = match message {
                                        IpcMessage::GetTabs => {
                                            info!("=== IPC SERVER: GetTabs request received ===");
                                            // Check if there's a pending command
                                            if let Some(command) = tab_manager.pop_command() {
                                                use my_launcher::core::native_messaging::ChromeCommand;
                                                use my_launcher::ipc::ChromeExtensionCommand;
                                                
                                                info!("Found pending command in queue!");
                                                let ext_command = match command {
                                                    ChromeCommand::SwitchToTab { tab_id, window_id } => {
                                                        info!("Command: SwitchToTab(tab_id={}, window_id={})", tab_id, window_id);
                                                        ChromeExtensionCommand::SwitchToTab { tab_id, window_id }
                                                    }
                                                };
                                                
                                                info!("Sending Chrome command to native host: {:?}", ext_command);
                                                IpcMessage::ChromeCommand { command: ext_command }
                                            } else {
                                                // No pending command, return tab list
                                                info!("No pending commands, returning tab list");
                                                let tabs = tab_manager.get_tabs();
                                                info!("Current tab count: {}", tabs.len());
                                                let ipc_tabs: Vec<TabInfo> = tabs.into_iter().map(|tab| TabInfo {
                                                    id: tab.id,
                                                    window_id: tab.window_id,
                                                    title: tab.title,
                                                    url: tab.url,
                                                    fav_icon_url: tab.fav_icon_url,
                                                    active: tab.active,
                                                    index: tab.index,
                                                }).collect();
                                                
                                                IpcMessage::TabList { tabs: ipc_tabs }
                                            }
                                        }
                                        IpcMessage::SwitchToTab { tab_id, window_id } => {
                                            // TODO: Implement actual tab switching
                                            // For now, just return success
                                            info!("Tab switch requested: tab_id={}, window_id={}", tab_id, window_id);
                                            IpcMessage::TabSwitchResult {
                                                success: true,
                                                error: None,
                                            }
                                        }
                                        IpcMessage::TabList { tabs } => {
                                            // Update the TabManager with the new tab list
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
                                            
                                            info!("Updating TabManager with {} tabs", chrome_tabs.len());
                                            tab_manager.update_tabs(chrome_tabs);
                                            
                                            // Send acknowledgment
                                            IpcMessage::TabSwitchResult {
                                                success: true,
                                                error: None,
                                            }
                                        }
                                        _ => {
                                            error!("Unexpected message type");
                                            continue;
                                        }
                                    };
                                    
                                    if let Err(e) = ipc::send_message(&mut server, &response).await {
                                        error!("Failed to send response: {}", e);
                                        break;
                                    }
                                    
                                    // Named pipes don't need explicit flush
                                }
                                Err(e) => {
                                    error!("Failed to read message: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to create IPC server: {}", e);
                // Wait before retrying
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let _ = my_launcher::logger::init_logger();

    // Create a shared TabManager instance
    let tab_manager = Arc::new(TabManager::new());
    
    // Start IPC server in a background thread on Windows
    #[cfg(windows)]
    {
        let tab_manager_clone = Arc::clone(&tab_manager);
        thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(run_ipc_server(tab_manager_clone));
        });
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_always_on_top()
            .with_decorations(true)
            .with_title("My Launcher - Alt+Tab Style"),
        ..Default::default()
    };

    let tab_manager_for_app = Arc::clone(&tab_manager);
    eframe::run_native(
        "My Launcher",
        options,
        Box::new(move |_cc| Box::new(LauncherApp::new_with_tab_manager(tab_manager_for_app))),
    )
}