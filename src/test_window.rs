use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // コンソールに直接出力
    eprintln!("=== Test Window Starting ===");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_decorations(true)
            .with_transparent(false),
        ..Default::default()
    };
    
    eprintln!("Creating window...");
    
    eframe::run_native(
        "Test Window",
        options,
        Box::new(|_cc| {
            eprintln!("App creator called!");
            Box::new(TestApp::default())
        }),
    )
}

#[derive(Default)]
struct TestApp {
    counter: u64,
}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.counter += 1;
        
        if self.counter == 1 {
            eprintln!("First frame!");
        }
        
        if self.counter % 60 == 0 {
            eprintln!("Frame: {}", self.counter);
        }
        
        ctx.request_repaint();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Test Window");
            ui.label(format!("Frame counter: {}", self.counter));
            
            if ui.button("Exit").clicked() {
                eprintln!("Exit clicked");
                std::process::exit(0);
            }
        });
    }
}