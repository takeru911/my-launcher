use eframe::egui;
use my_launcher::{
    ui::alt_tab_grid::AltTabGrid,
    data::{
        window_provider::{WindowProvider, WindowsApiProvider},
        window_item::WindowItem,
    },
    filter::{WindowFilter, TaskbarWindowFilter, filter_windows, SearchFilter, search_items},
    ThumbnailCache,
};
use std::error::Error;

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

struct AltTabApp {
    /// グリッドUIコンポーネント
    grid: AltTabGrid,
    /// ウィンドウ情報プロバイダー
    window_provider: Box<dyn WindowProvider>,
    /// 検索テキスト
    search_text: String,
    /// 検索バーの表示フラグ
    show_search_bar: bool,
    /// フィルタリングされたウィンドウリスト
    filtered_windows: Vec<WindowItem>,
    /// サムネイルキャッシュ
    thumbnail_cache: ThumbnailCache,
    /// ウィンドウフィルタ
    window_filter: Box<dyn WindowFilter>,
}

impl AltTabApp {
    fn new() -> Self {
        let mut window_provider = Box::new(WindowsApiProvider::new());
        window_provider.refresh();
        
        let window_filter = Box::new(TaskbarWindowFilter::new());
        let all_windows = window_provider.get_windows();
        let filtered = filter_windows(all_windows, window_filter.as_ref());
        
        log::info!("Total windows: {}, Taskbar windows: {}", 
                  window_provider.get_windows().len(), 
                  filtered.len());
        
        Self {
            grid: AltTabGrid::new(),
            window_provider,
            search_text: String::new(),
            show_search_bar: true,  // デフォルトで検索バーを表示
            filtered_windows: filtered,
            thumbnail_cache: ThumbnailCache::new(),
            window_filter,
        }
    }

    fn filter_windows(&mut self) {
        let all_windows = self.window_provider.get_windows();
        
        // まずタスクバーフィルタを適用
        let taskbar_windows = filter_windows(all_windows, self.window_filter.as_ref());
        
        // 次に検索フィルタを適用
        let search_filter = SearchFilter::new(&self.search_text);
        self.filtered_windows = search_items(taskbar_windows, &search_filter);
        
        // 選択インデックスを調整
        if self.grid.selected_index >= self.filtered_windows.len() {
            self.grid.selected_index = self.filtered_windows.len().saturating_sub(1);
        }
    }

    fn switch_to_selected_window(&self) {
        if self.grid.selected_index < self.filtered_windows.len() {
            let window = &self.filtered_windows[self.grid.selected_index];
            if let Err(e) = self.window_provider.focus_window(window.hwnd) {
                log::error!("Failed to switch window: {}", e);
            } else {
                log::info!("Switched to window: {}", window.title);
            }
        }
    }
}

impl eframe::App for AltTabApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ダークテーマを設定
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = egui::Color32::from_rgba_premultiplied(30, 30, 30, 240);
        ctx.set_visuals(visuals);
        
        // 継続的な再描画をリクエスト
        ctx.request_repaint();

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 240)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // キーボードショートカットの処理
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        log::info!("Escape pressed, exiting");
                        std::process::exit(0);
                    }
                    
                    if ui.input(|i| i.key_pressed(egui::Key::F1)) {
                        self.show_search_bar = !self.show_search_bar;
                        log::debug!("Search bar toggled: {}", self.show_search_bar);
                    }
                    
                    if ui.input(|i| i.key_pressed(egui::Key::F5)) {
                        self.window_provider.refresh();
                        self.filter_windows();
                        log::debug!("Windows refreshed");
                    }
                    
                    // 検索バー
                    if self.show_search_bar {
                        ui.add_space(20.0);
                        
                        ui.horizontal(|ui| {
                            ui.add_space((ui.available_width() - 600.0) / 2.0);
                            
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut self.search_text)
                                    .desired_width(600.0)
                                    .font(egui::TextStyle::Heading)
                                    .hint_text("Search windows...")
                                    .frame(true)
                            );
                            
                            if response.changed() {
                                self.filter_windows();
                            }
                            
                            response.request_focus();
                        });
                        
                        ui.add_space(10.0);
                        
                        // 検索結果数を表示
                        if !self.filtered_windows.is_empty() {
                            ui.label(
                                egui::RichText::new(format!("{} windows", self.filtered_windows.len()))
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(150, 150, 150))
                            );
                        }
                        
                        ui.add_space(20.0);
                    }
                    
                    // ウィンドウグリッド
                    if !self.filtered_windows.is_empty() {
                        // キーボードナビゲーション
                        self.grid.handle_keyboard_navigation(ui, self.filtered_windows.len());
                        
                        // グリッド表示
                        if let Some(clicked_index) = self.grid.show(ui, ctx, &self.filtered_windows, &mut self.thumbnail_cache) {
                            if clicked_index < self.filtered_windows.len() {
                                let window = &self.filtered_windows[clicked_index];
                                log::info!("Window clicked: {}", window.title);
                                if let Err(e) = self.window_provider.focus_window(window.hwnd) {
                                    log::error!("Failed to switch window: {}", e);
                                }
                                std::process::exit(0);
                            }
                        }
                        
                        // Enter キーで選択
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.switch_to_selected_window();
                            std::process::exit(0);
                        }
                    } else {
                        ui.add_space(100.0);
                        ui.label(
                            egui::RichText::new("No windows found")
                                .size(16.0)
                                .color(egui::Color32::from_rgb(150, 150, 150))
                        );
                    }
                    
                    // 操作説明
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add_space(20.0);
                        ui.label(
                            egui::RichText::new("↑↓←→ Navigate • Enter Switch • F1 Search • F5 Refresh • Esc Exit")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(120, 120, 120))
                        );
                    });
                });
            });
    }
}

fn main() -> Result<(), eframe::Error> {
    // ロガーを初期化
    if let Err(e) = my_launcher::logger::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    }
    
    log::info!("Starting Alt+Tab style launcher...");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false),
        vsync: true,
        centered: true,
        ..Default::default()
    };
    
    eframe::run_native(
        "Alt+Tab Launcher",
        options,
        Box::new(|cc| {
            log::info!("Creating AltTabApp instance");
            
            // Try to load Japanese font at runtime
            if let Err(e) = setup_custom_fonts(&cc.egui_ctx) {
                log::warn!("Failed to setup custom fonts: {}", e);
            }
            
            Box::new(AltTabApp::new())
        }),
    )
}