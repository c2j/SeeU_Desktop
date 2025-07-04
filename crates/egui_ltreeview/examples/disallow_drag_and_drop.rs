#[path = "data.rs"]
mod data;

// use egui::ThemePreference; // Not available in egui 0.28.1
use egui_ltreeview::TreeView;

fn main() -> Result<(), eframe::Error> {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Egui_ltreeview simple example",
        options,
        Box::new(|cc| {
            // Set dark theme - in egui 0.28.1, use set_visuals instead
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::<MyApp>::default())
        }),
    )
}

#[derive(Default)]
struct MyApp {}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            TreeView::new(ui.make_persistent_id("Names tree view"))
                .allow_drag_and_drop(false)
                .show(ui, |builder| {
                    builder.dir(0, "Root");
                    builder.dir(1, "Foo");
                    builder.leaf(2, "Ava");
                    builder.dir(3, "Bar");
                    builder.leaf(4, "Benjamin");
                    builder.leaf(5, "Charlotte");
                    builder.close_dir();
                    builder.close_dir();
                    builder.leaf(6, "Daniel");
                    builder.leaf(7, "Emma");
                    builder.dir(8, "Baz");
                    builder.leaf(9, "Finn");
                    builder.leaf(10, "Grayson");
                    builder.close_dir();
                    builder.close_dir();
                });
        });
    }
}
