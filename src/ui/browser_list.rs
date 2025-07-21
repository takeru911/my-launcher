use egui;
use crate::core::search_engine::{SearchResult, ResultType};

pub struct BrowserList {
    pub selected_index: usize,
    visible_items: usize,
    items_per_batch: usize,
}

impl BrowserList {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            visible_items: 20,  // 初期表示数
            items_per_batch: 10, // スクロール時の追加表示数
        }
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        results: &[SearchResult],
    ) -> Option<usize> {
        let mut clicked_index = None;
        
        // 選択されたアイテムが表示範囲に近い場合、表示数を増やす
        if self.selected_index + 5 >= self.visible_items && self.visible_items < results.len() {
            self.visible_items = (self.visible_items + self.items_per_batch).min(results.len());
        }
        
        // 表示する結果を制限
        let display_results = &results[..self.visible_items.min(results.len())];

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (index, result) in display_results.iter().enumerate() {
                let is_selected = index == self.selected_index;
                
                ui.horizontal(|ui| {
                    // 選択状態の表示
                    if is_selected {
                        ui.label("▶");
                    } else {
                        ui.label(" ");
                    }
                    
                    // 結果タイプに応じたアイコン
                    let icon = match &result.result_type {
                        ResultType::GoogleSearch => "🔍",
                        ResultType::Bookmark => "⭐",
                        ResultType::History => "🕒",
                        ResultType::Window => "🪟",
                        ResultType::Tab => "📑",
                    };
                    ui.label(icon);
                    
                    // タイトルと説明を縦に並べて表示
                    ui.vertical(|ui| {
                        // 結果タイプに応じた背景色
                        let bg_color = match &result.result_type {
                            ResultType::GoogleSearch => egui::Color32::from_rgb(40, 50, 60),  // 青っぽい
                            ResultType::Bookmark => egui::Color32::from_rgb(60, 50, 40),      // 黄色っぽい
                            ResultType::History => egui::Color32::from_rgb(50, 40, 50),       // 紫っぽい
                            ResultType::Window => egui::Color32::from_rgb(40, 40, 40),        // グレー
                            ResultType::Tab => egui::Color32::from_rgb(40, 60, 40),          // 緑っぽい
                        };
                        
                        let selected_bg_color = match &result.result_type {
                            ResultType::GoogleSearch => egui::Color32::from_rgb(50, 70, 90),
                            ResultType::Bookmark => egui::Color32::from_rgb(90, 70, 50),
                            ResultType::History => egui::Color32::from_rgb(70, 50, 70),
                            ResultType::Window => egui::Color32::from_rgb(60, 60, 60),
                            ResultType::Tab => egui::Color32::from_rgb(50, 80, 50),
                        };
                        
                        let response = ui.add(
                            egui::Button::new(&result.title)
                                .fill(if is_selected { selected_bg_color } else { bg_color })
                                .min_size(egui::Vec2::new(ui.available_width() - 20.0, 30.0))
                        );
                        
                        if response.clicked() {
                            clicked_index = Some(index);
                        }
                        
                        if is_selected {
                            response.scroll_to_me(Some(egui::Align::Center));
                        }
                        
                        // 説明文を表示（履歴の場合はURLをトリミング）
                        let description = if matches!(result.result_type, ResultType::History) {
                            Self::trim_url_for_display(&result.description, 50)
                        } else {
                            result.description.clone()
                        };
                        
                        ui.label(
                            egui::RichText::new(&description)
                                .small()
                                .color(egui::Color32::from_gray(180))
                        );
                    });
                });
                
                ui.add_space(5.0);
            }
        });

        clicked_index
    }

    fn trim_url_for_display(url: &str, max_query_length: usize) -> String {
        // URLとvisit count情報を分離
        let (url_part, visit_info) = if let Some(pos) = url.rfind(" (visited") {
            (&url[..pos], &url[pos..])
        } else {
            (url, "")
        };

        // クエリパラメータをトリミング
        let trimmed_url = if let Some(query_start) = url_part.find('?') {
            let base = &url_part[..query_start];
            let query = &url_part[query_start..];
            
            if query.len() > max_query_length {
                format!("{}{}...", base, &query[..max_query_length])
            } else {
                url_part.to_string()
            }
        } else {
            url_part.to_string()
        };

        // visit count情報を再度追加
        format!("{}{}", trimmed_url, visit_info)
    }

    pub fn handle_keyboard(&mut self, ui: &mut egui::Ui, results_count: usize) {
        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            if self.selected_index < results_count.saturating_sub(1) {
                self.selected_index += 1;
                
                // 下にスクロールするときに表示数を増やす
                if self.selected_index + 5 >= self.visible_items && self.visible_items < results_count {
                    self.visible_items = (self.visible_items + self.items_per_batch).min(results_count);
                }
            }
        }

        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }

        if ui.input(|i| i.key_pressed(egui::Key::Home)) {
            self.selected_index = 0;
        }

        if ui.input(|i| i.key_pressed(egui::Key::End)) && results_count > 0 {
            self.selected_index = results_count - 1;
            // 最後まで表示
            self.visible_items = results_count;
        }
    }
    
    pub fn reset_for_new_search(&mut self) {
        self.selected_index = 0;
        self.visible_items = 20;
    }
}