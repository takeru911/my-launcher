use egui::{Vec2, Pos2, Rect, Color32, Stroke, Rounding, Sense};
use crate::ThumbnailCache;

/// Alt+Tab風のグリッド表示のためのUIコンポーネント
pub struct AltTabGrid {
    /// 選択中のアイテムのインデックス
    pub selected_index: usize,
    /// グリッドの列数
    pub columns: usize,
    /// 各アイテムのサイズ
    pub item_size: Vec2,
    /// アイテム間のスペース
    pub spacing: f32,
}

/// グリッドに表示するアイテムのインターフェース
pub trait GridItem {
    /// アイテムのタイトル
    fn title(&self) -> &str;
    /// アイテムの説明（プロセス名など）
    fn description(&self) -> &str;
    /// ウィンドウハンドル（サムネイル取得用）
    fn hwnd(&self) -> isize;
    /// アイテムの一意な識別子
    fn id(&self) -> String;
}

impl AltTabGrid {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            columns: 5,
            item_size: Vec2::new(200.0, 150.0),
            spacing: 10.0,
        }
    }

    /// グリッドを表示し、クリックされたアイテムのインデックスを返す
    pub fn show<T: GridItem>(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        items: &[T],
        thumbnail_cache: &mut ThumbnailCache,
    ) -> Option<usize> {
        if items.is_empty() {
            return None;
        }

        let mut clicked_index = None;

        // グリッドの寸法を計算
        let total_width = self.columns as f32 * (self.item_size.x + self.spacing) - self.spacing;
        let rows = (items.len() + self.columns - 1) / self.columns;
        let total_height = rows as f32 * (self.item_size.y + self.spacing) - self.spacing;

        // グリッドを中央に配置
        let available_rect = ui.available_rect_before_wrap();
        let grid_rect = Rect::from_min_size(
            Pos2::new(
                available_rect.center().x - total_width / 2.0,
                available_rect.min.y + 20.0,
            ),
            Vec2::new(total_width, total_height),
        );

        ui.allocate_rect(grid_rect, Sense::hover());

        // 各アイテムを描画
        for (index, item) in items.iter().enumerate() {
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

            // 背景を描画
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

            // 選択されているアイテムのボーダーを描画
            if is_selected {
                ui.painter().rect_stroke(
                    item_rect,
                    Rounding::same(8.0),
                    Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
                );
            }

            // 内側のパディング
            let inner_rect = Rect::from_min_size(
                item_rect.min + Vec2::new(10.0, 10.0),
                item_rect.size() - Vec2::new(20.0, 20.0),
            );

            // サムネイル/アイコンエリア
            let thumbnail_size = Vec2::new(inner_rect.width(), inner_rect.height() - 40.0);
            let thumbnail_rect = Rect::from_min_size(inner_rect.min, thumbnail_size);

            // サムネイルまたはプレースホルダーを描画
            // 高解像度でキャプチャして、表示時にスケールダウン
            let capture_size = (
                (thumbnail_rect.width() * 2.0) as u32,
                (thumbnail_rect.height() * 2.0) as u32,
            );
            if let Some(texture) = thumbnail_cache.get_or_create_thumbnail(
                ctx,
                item.hwnd(),
                capture_size,
            ) {
                ui.put(
                    thumbnail_rect,
                    egui::Image::from_texture(texture)
                        .fit_to_exact_size(thumbnail_rect.size())
                        .rounding(Rounding::same(4.0)),
                );
            } else {
                self.draw_placeholder(ui, thumbnail_rect);
            }

            // タイトルを描画
            let title_rect = Rect::from_min_size(
                Pos2::new(inner_rect.min.x, inner_rect.max.y - 35.0),
                Vec2::new(inner_rect.width(), 20.0),
            );

            let title_text = self.truncate_text(item.title(), 25);
            ui.painter().text(
                title_rect.center(),
                egui::Align2::CENTER_CENTER,
                title_text,
                egui::FontId::proportional(14.0),
                Color32::from_rgb(220, 220, 220),
            );

            // 説明を描画
            let desc_rect = Rect::from_min_size(
                Pos2::new(inner_rect.min.x, inner_rect.max.y - 15.0),
                Vec2::new(inner_rect.width(), 15.0),
            );

            ui.painter().text(
                desc_rect.center(),
                egui::Align2::CENTER_CENTER,
                item.description(),
                egui::FontId::proportional(11.0),
                Color32::from_rgb(150, 150, 150),
            );

            // インタラクション処理
            let response = ui.interact(item_rect, ui.id().with(index), Sense::click());

            if response.clicked() {
                clicked_index = Some(index);
            }

            if response.hovered() {
                self.selected_index = index;
                ui.ctx().request_repaint();
            }
        }

        clicked_index
    }

    /// キーボードナビゲーションを処理
    pub fn handle_keyboard_navigation(&mut self, ui: &egui::Ui, item_count: usize) {
        if item_count == 0 {
            return;
        }

        let input = ui.input(|i| i.clone());

        if input.key_pressed(egui::Key::ArrowRight) {
            self.selected_index = (self.selected_index + 1) % item_count;
        }

        if input.key_pressed(egui::Key::ArrowLeft) {
            self.selected_index = if self.selected_index == 0 {
                item_count - 1
            } else {
                self.selected_index - 1
            };
        }

        if input.key_pressed(egui::Key::ArrowDown) {
            let new_index = self.selected_index + self.columns;
            if new_index < item_count {
                self.selected_index = new_index;
            }
        }

        if input.key_pressed(egui::Key::ArrowUp) {
            if self.selected_index >= self.columns {
                self.selected_index -= self.columns;
            }
        }

        if input.key_pressed(egui::Key::Home) {
            self.selected_index = 0;
        }

        if input.key_pressed(egui::Key::End) {
            self.selected_index = item_count - 1;
        }
    }

    /// プレースホルダーを描画
    fn draw_placeholder(&self, ui: &egui::Ui, rect: Rect) {
        ui.painter().rect_filled(
            rect,
            Rounding::same(4.0),
            Color32::from_rgb(30, 30, 30),
        );

        // ウィンドウアイコンのプレースホルダー
        let icon_size = 48.0;
        let icon_rect = Rect::from_center_size(
            rect.center(),
            Vec2::splat(icon_size),
        );
        ui.painter().rect_stroke(
            icon_rect,
            Rounding::same(4.0),
            Stroke::new(2.0, Color32::from_rgb(80, 80, 80)),
        );
    }

    /// テキストを指定された長さで切り詰める
    fn truncate_text(&self, text: &str, max_len: usize) -> String {
        if text.chars().count() > max_len {
            let truncated: String = text.chars().take(max_len - 3).collect();
            format!("{}...", truncated)
        } else {
            text.to_string()
        }
    }
}

impl Default for AltTabGrid {
    fn default() -> Self {
        Self::new()
    }
}