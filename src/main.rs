#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hides console window on Windows in release

fn main() {
    #[cfg(not(debug_assertions))]
    run_shears_version_check_thread();

    if let Err(e) = shears_main() {
        win_msgbox::error::<win_msgbox::Okay>(&e.to_string())
            .show()
            .expect("Failed to show messagebox");
    }
}

fn shears_main() -> eframe::Result {
    let window_size = [600.0, 450.0];

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
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

pub fn run_shears_version_check_thread() {
    std::thread::spawn(|| {
        let response = minreq::get("http://api.github.com/repos/lungu19/shears/releases/latest")
            .with_header("User-Agent", "shears-update-check")
            .send()
            .expect("Failed to send rquest");

        if let Ok(raw_json) = &response.as_str() {
            let json = microjson::JSONValue::load(raw_json);

            if let Ok(version_value) = json.get_key_value("tag_name") {
                if let Ok(version_string) = version_value.read_string() {
                    let current_version = env!("CARGO_PKG_VERSION");

                    log::info!("current_version: {current_version}");
                    log::info!("version_string: {version_string}");

                    if current_version == version_string {
                        log::info!("Shears is up-to-date");
                        return;
                    }

                    log::info!("Shears is not up-to-date");
                    match win_msgbox::warning::<win_msgbox::OkayCancel>(
                        "A newer version of Shears is available. Do you want to update now?",
                    )
                    .show()
                    .expect("Failed to show messagebox")
                    {
                        win_msgbox::OkayCancel::Okay => {
                            open::that("https://github.com/lungu19/shears/releases/latest")
                                .expect("Failed to open link in browser");
                        }
                        win_msgbox::OkayCancel::Cancel => {}
                    }
                }
            }
        } else {
            log::warn!("Failed to get json value");
        }
    });
}
