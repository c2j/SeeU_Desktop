#[path = "data.rs"]
mod data;

// use egui::ThemePreference; // Not available in egui 0.28.1
use egui_ltreeview::{NodeBuilder, TreeView};

fn main() -> Result<(), eframe::Error> {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 500.0]),

        ..Default::default()
    };
    eframe::run_native(
        "Egui_ltreeview example",
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
            TreeView::new(ui.make_persistent_id("Names tree view")).show(ui, |builder| {
                builder.node(NodeBuilder::dir(0).default_open(false).label("root"));

                builder.node(NodeBuilder::dir(1).default_open(false).label("Foo"));
                builder.leaf(2, "Ava");
                builder.node(NodeBuilder::dir(3).default_open(false).label("Bar"));
                builder.leaf(4, "Benjamin");
                builder.leaf(5, "Charlotte");
                builder.close_dir();
                builder.close_dir();
                builder.leaf(6, "Daniel");
                builder.leaf(7, "Emma");
                builder.node(NodeBuilder::dir(8).default_open(false).label("Baz"));
                builder.leaf(9, "Finn");
                builder.leaf(10, "Grayson");
                builder.close_dir();
                builder.close_dir();
            });
        });
    }
}
