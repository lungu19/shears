#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hides console window on Windows in release

fn main() {
    if let Err(e) = shears_main() {
        native_dialog::DialogBuilder::message()
            .set_level(native_dialog::MessageLevel::Error)
            .set_text(e.to_string())
            .alert()
            .show()
            .expect("Failed to show dialog");
    }
}

fn shears_main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let window_size = [600.0, 450.0];

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("Shears")
            .with_inner_size(window_size)
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                    .expect("Failed to load icon"),
            )
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };

    eframe::run_native(
        format!("Shears {}", env!("CARGO_PKG_VERSION")).as_str(),
        native_options,
        Box::new(|cc| Ok(Box::new(shears::ShearsApp::new(cc)))),
    )
}
