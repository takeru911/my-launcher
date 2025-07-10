use egui::{Rect, Sense, Vec2, Pos2, Color32, Stroke, Rounding};
use crate::{SearchResult, ThumbnailCache};

pub struct WindowGrid {
    pub selected_index: usize,
    pub columns: usize,
    pub item_size: Vec2,
    pub spacing: f32,
}

impl WindowGrid {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            columns: 5,  // 5 columns like Windows Alt+Tab
            item_size: Vec2::new(200.0, 150.0),
            spacing: 10.0,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        results: &[SearchResult],
        thumbnail_cache: &mut ThumbnailCache,
    ) -> Option<usize> {
        let mut clicked_index = None;
        
        // Calculate grid dimensions
        let total_width = self.columns as f32 * (self.item_size.x + self.spacing) - self.spacing;
        let rows = (results.len() + self.columns - 1) / self.columns;
        let total_height = rows as f32 * (self.item_size.y + self.spacing) - self.spacing;
        
        // Center the grid
        let available_rect = ui.available_rect_before_wrap();
        let grid_rect = Rect::from_min_size(
            Pos2::new(
                available_rect.center().x - total_width / 2.0,
                available_rect.min.y + 20.0,
            ),
            Vec2::new(total_width, total_height),
        );

        ui.allocate_rect(grid_rect, Sense::hover());

        for (index, result) in results.iter().enumerate() {
            let row = index / self.columns;
            let col = index % self.columns;
            
            let item_rect = Rect::from_min_size(
                Pos2::new(
                    grid_rect.min.x + col as f32 * (self.item_size.x + self.spacing),
                    grid_rect.min.y + row as f32 * (self.item_size.y + self.spacing),
                ),
                self.item_size,
            );

            let is_selected = index == self.selected_index;
            
            // Draw background
            let bg_color = if is_selected {
                Color32::from_rgb(60, 60, 60)
            } else {
                Color32::from_rgb(40, 40, 40)
            };
            
            ui.painter().rect_filled(
                item_rect,
                Rounding::same(8.0),
                bg_color,
            );
            
            // Draw border for selected item
            if is_selected {
                ui.painter().rect_stroke(
                    item_rect,
                    Rounding::same(8.0),
                    Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
                );
            }
            
            // Inner padding
            let inner_rect = Rect::from_min_size(
                item_rect.min + Vec2::new(10.0, 10.0),
                item_rect.size() - Vec2::new(20.0, 20.0),
            );
            
            // Draw thumbnail
            if let Some(window_info) = &result.window_info {
                let thumbnail_size = Vec2::new(inner_rect.width(), inner_rect.height() - 40.0);
                let thumbnail_rect = Rect::from_min_size(inner_rect.min, thumbnail_size);
                
                if let Some(texture) = thumbnail_cache.get_or_create_thumbnail(
                    ctx,
                    window_info.hwnd,
                    (thumbnail_size.x as u32, thumbnail_size.y as u32),
                ) {
                    ui.painter().image(
                        texture.id(),
                        thumbnail_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    // Placeholder for missing thumbnail
                    ui.painter().rect_filled(
                        thumbnail_rect,
                        Rounding::same(4.0),
                        Color32::from_rgb(30, 30, 30),
                    );
                    
                    // Draw a window icon placeholder
                    let icon_size = 48.0;
                    let icon_rect = Rect::from_center_size(
                        thumbnail_rect.center(),
                        Vec2::splat(icon_size),
                    );
                    ui.painter().rect_stroke(
                        icon_rect,
                        Rounding::same(4.0),
                        Stroke::new(2.0, Color32::from_rgb(80, 80, 80)),
                    );
                }
                
                // Draw title
                let title_rect = Rect::from_min_size(
                    Pos2::new(inner_rect.min.x, inner_rect.max.y - 35.0),
                    Vec2::new(inner_rect.width(), 35.0),
                );
                
                let title_text = if result.title.len() > 25 {
                    format!("{}...", &result.title[..22])
                } else {
                    result.title.clone()
                };
                
                ui.painter().text(
                    title_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    title_text,
                    egui::FontId::proportional(14.0),
                    Color32::from_rgb(220, 220, 220),
                );
                
                // Draw process name in smaller text
                let process_text = window_info.process_name.clone();
                ui.painter().text(
                    Pos2::new(title_rect.center().x, title_rect.max.y - 5.0),
                    egui::Align2::CENTER_TOP,
                    process_text,
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(150, 150, 150),
                );
            }
            
            // Handle interaction
            let response = ui.interact(item_rect, ui.id().with(index), Sense::click_and_drag());
            
            if response.clicked() {
                clicked_index = Some(index);
            }
            
            if response.hovered() {
                self.selected_index = index;
                ui.ctx().request_repaint(); // Ensure hover updates are smooth
            }
        }
        
        clicked_index
    }
    
    pub fn handle_keyboard_navigation(&mut self, ui: &egui::Ui, results_count: usize) {
        if results_count == 0 {
            return;
        }
        
        let input = ui.input(|i| i.clone());
        
        if input.key_pressed(egui::Key::ArrowRight) {
            self.selected_index = (self.selected_index + 1) % results_count;
        }
        
        if input.key_pressed(egui::Key::ArrowLeft) {
            self.selected_index = if self.selected_index == 0 {
                results_count - 1
            } else {
                self.selected_index - 1
            };
        }
        
        if input.key_pressed(egui::Key::ArrowDown) {
            let new_index = self.selected_index + self.columns;
            if new_index < results_count {
                self.selected_index = new_index;
            }
        }
        
        if input.key_pressed(egui::Key::ArrowUp) {
            if self.selected_index >= self.columns {
                self.selected_index -= self.columns;
            }
        }
        
        // Home/End keys
        if input.key_pressed(egui::Key::Home) {
            self.selected_index = 0;
        }
        
        if input.key_pressed(egui::Key::End) {
            self.selected_index = results_count - 1;
        }
    }
}