use std::path::PathBuf;

use crate::{
    helpers::{
        delete_events_folder, delete_texture_files, delete_videos_folder,
        get_shearing_features_availability, is_siege_running, run_shears_version_background_check,
        write_streaminginstall,
    },
    settings::PersistentSettingsStorage,
    types::{
        ForgeTextureQualityLevel, ShearsFolderState, ShearsModals, ShearsPage,
        ShearsScanFolderState, ShearsUiState,
    },
};

#[derive(Default)]
pub struct ShearsApp {
    folder_state: ShearsFolderState,
    scan_state: ShearsScanFolderState,
    ui_state: ShearsUiState,
    system_information: sysinfo::System,
    persistent_settings_storage: PersistentSettingsStorage,
}

impl ShearsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        catppuccin_egui::set_theme(&cc.egui_ctx, catppuccin_egui::MACCHIATO);

        cc.egui_ctx.all_styles_mut(|style| {
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(30);
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(30);
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(30);
            style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
            style.visuals.widgets.open.corner_radius = egui::CornerRadius::same(30);

            style.spacing.button_padding = egui::vec2(5., 2.5);
        });

        let settings = PersistentSettingsStorage::load_or_default();

        if settings.enable_shears_update_check_on_startup {
            run_shears_version_background_check(true);
        }

        Self {
            system_information: sysinfo::System::new(),
            persistent_settings_storage: settings,
            ..Self::default()
        }
    }

    pub fn set_folder(&mut self, folder: &std::path::Path) {
        self.folder_state.siege_path = Some(folder.to_path_buf());
        self.refresh_feature_availablity();
    }

    pub fn show_scan_drives_page(&mut self) {
        self.scan_state.update_disks();
        self.ui_state.change_page(ShearsPage::DiskScanSelect);
    }

    pub fn refresh_feature_availablity(&mut self) {
        let siege_path =
            self.folder_state.siege_path.as_ref().expect(
                "ShearsApp.refresh_feature_availablity: Failed to get folder_state.siege_path",
            );
        self.folder_state.features_availability = get_shearing_features_availability(siege_path);

        self.ui_state.change_page(ShearsPage::FolderSelected);

        // set the feature checkboxes accordingly
        for quality_level in
            ForgeTextureQualityLevel::Low as usize..=ForgeTextureQualityLevel::Ultra as usize
        {
            *self.ui_state.get_texture_checkbox_mut(quality_level) = self
                .folder_state
                .features_availability
                .get_texture(quality_level)
                .0;
        }

        self.folder_state
            .features_availability
            .get_texture_mut(ForgeTextureQualityLevel::Low as usize)
            .0 = false; // let's avoid users making their games completely unplayable

        self.ui_state.checkbox_videos = self.folder_state.features_availability.videos.0;
        self.ui_state.checkbox_events = self.folder_state.features_availability.events.0;

        self.compute_possible_space_freed();
    }

    fn compute_possible_space_freed(&mut self) {
        self.ui_state.label_possible_space_saved = 0;

        for quality_level in
            ForgeTextureQualityLevel::Medium as usize..=ForgeTextureQualityLevel::Ultra as usize
        // the user is not able to remove low textures so we can just skip it here
        {
            if !self.ui_state.get_texture_checkbox(quality_level) {
                self.ui_state.label_possible_space_saved += self
                    .folder_state
                    .features_availability
                    .get_texture(quality_level)
                    .1;
            }
        }

        if !self.ui_state.checkbox_videos {
            self.ui_state.label_possible_space_saved +=
                self.folder_state.features_availability.videos.1;
        }

        if !self.ui_state.checkbox_events {
            self.ui_state.label_possible_space_saved +=
                self.folder_state.features_availability.events.1;
        }
    }

    fn execute_shearing(&mut self) -> bool {
        let mut min_texture_quality_level = ForgeTextureQualityLevel::Low;
        for i in (ForgeTextureQualityLevel::Medium.convert_to_i32()
            ..=ForgeTextureQualityLevel::Ultra.convert_to_i32())
            .rev()
        {
            let level = ForgeTextureQualityLevel::convert_from_i32(i)
                .expect("Failed to convert i32 to ForgeTextureQualityLevel");
            if self.ui_state.get_texture_checkbox(level as usize) {
                min_texture_quality_level = level;
                break;
            }
        }

        if let Some(path_str) = self.folder_state.siege_path.clone() {
            let path = std::path::Path::new(&path_str);

            delete_texture_files(path, &min_texture_quality_level);

            if !self.ui_state.checkbox_videos {
                delete_videos_folder(path);
            }

            if !self.ui_state.checkbox_events {
                delete_events_folder(path);
            }

            write_streaminginstall(path).expect("Failed to write streaming install");

            self.set_folder(path);

            return true;
        }

        false
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Siege folder").clicked()
                        && let Some(path) = rfd::FileDialog::new().pick_folder()
                    {
                        log::info!("Clicked on `Open Siege folder` button");
                        self.set_folder(&path);
                    }

                    if ui.button("Settings").clicked() {
                        log::info!("Clicked on `Settings` button");
                        *self.ui_state.get_modal_mut(ShearsModals::Settings as usize) = true;
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        log::info!("Clicked on `Quit` button");
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Check for updates").clicked() {
                        run_shears_version_background_check(false);
                    }
                    if ui.button("About").clicked() {
                        *self.ui_state.get_modal_mut(ShearsModals::About as usize) = true;
                    }
                });

                #[cfg(debug_assertions)]
                {
                    ui.label(
                        egui::RichText::new("Debug build")
                            .small()
                            .color(ui.visuals().error_fg_color),
                    )
                    .on_hover_text("compiled with debug assertions enabled.");

                    ui.label(
                        egui::RichText::new(format!("Page: {:?}", self.ui_state.get_page()))
                            .small()
                            .color(ui.visuals().warn_fg_color),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "LastPage: {:?}",
                            self.ui_state.get_last_page()
                        ))
                        .small()
                        .color(ui.visuals().warn_fg_color),
                    );
                }
            });
        });
    }

    fn render_main_content(&mut self, ctx: &egui::Context) {
        match self.ui_state.get_page() {
            ShearsPage::MainPage => self.render_main_page(ctx),
            ShearsPage::FolderSelected => self.render_folder_selected_page(ctx),
            ShearsPage::DiskScanSelect => self.render_disk_scan_select_page(ctx),
            ShearsPage::DiskScanInProgress => self.render_disk_scan_in_progress_page(ctx),
            ShearsPage::DiskScanComplete => self.render_disk_scan_complete_page(ctx),
        }
    }

    fn render_main_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()))
            .show(ctx, |ui| {
                let string = if self.persistent_settings_storage.use_loose_selection {
                    "Drag and drop or click to select the Siege folder..."
                } else {
                    "Drag and drop or click to select the Siege .exe file..."
                };

                let style = ui.style_mut();
                style.spacing.button_padding = egui::vec2(8.0, 5.0);

                let radius = egui::CornerRadius::same(4); // or egui::CornerRadius::ZERO for sharp corners
                style.visuals.widgets.noninteractive.corner_radius = radius;
                style.visuals.widgets.inactive.corner_radius = radius;
                style.visuals.widgets.hovered.corner_radius = radius;
                style.visuals.widgets.active.corner_radius = radius;
                style.visuals.widgets.open.corner_radius = radius;

                let spacing = ui.spacing().item_spacing.y;
                let total_height = ui.available_height();
                let top_height = (total_height - spacing) * (2.0 / 3.0);

                if ui
                    .add_sized(
                        [ui.available_width(), top_height],
                        egui::Button::new(string),
                    )
                    .clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_folder()
                {
                    self.set_folder(&path);
                }

                if ui
                    .add_sized(
                        ui.available_size(),
                        egui::Button::new("Click to scan your drive for Siege installations..."),
                    )
                    .clicked()
                {
                    self.show_scan_drives_page();
                }
            });
    }

    fn render_disk_scan_select_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Back").clicked() {
                        self.ui_state.go_back();
                    }
                });

                ui.heading("Select the Drive to scan");

                let mut selected_mount_point: Option<std::path::PathBuf> = None;

                for disk in &self.scan_state.disks {
                    let name = {
                        let name = disk.name();
                        if !name.is_empty() {
                            name.to_string_lossy().into_owned()
                        } else {
                            "Local Disk".to_owned()
                        }
                    };
                    let kind = disk.kind();
                    let mount_point = disk.mount_point();

                    if ui
                        .button(format!("{name} ({}) [{kind}]", mount_point.display()))
                        .clicked()
                    {
                        selected_mount_point = Some(mount_point.to_path_buf());
                    }
                }

                if let Some(mount_point) = selected_mount_point {
                    self.scan_state.start_scan_thread(mount_point);
                    self.ui_state.change_page(ShearsPage::DiskScanInProgress);
                }
            });
    }

    fn render_disk_scan_in_progress_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()))
            .show(ctx, |ui| {
                let is_finished = self
                    .scan_state
                    .thread_handle
                    .as_ref()
                    .is_some_and(|h| h.is_finished());

                if is_finished {
                    let handle = self.scan_state.thread_handle.take().expect(
                        "render_disk_scan_in_progress_page: Failed to unwrap thread handle",
                    ); // take handle and leave None

                    // capture thread return value
                    match handle.join() {
                        Ok(result) => {
                            log::info!("Completed disk scan, found {} items", result.len());
                            self.scan_state.scan_results = Some(result);
                            self.ui_state.change_page(ShearsPage::DiskScanComplete);
                        }
                        Err(e) => log::error!("Thread panicked: {e:?}"),
                    }
                }

                // thread is still going on
                if self.scan_state.thread_handle.is_some() {
                    ui.heading("Scanning drive for Old Siege instances...");
                    ui.label(format!("Time elapsed: {}", self.scan_state.timer_elapsed()));
                    ctx.request_repaint();
                }

                if ui.button("Stop scan").clicked() {
                    self.scan_state
                        .stop_flag
                        .store(true, std::sync::atomic::Ordering::Relaxed);

                    if let Some(handle) = self.scan_state.thread_handle.take() {
                        if let Err(e) = handle.join() {
                            log::error!("Scan thread panicked: {e:?}");
                        } else {
                            log::info!("Scan thread stopped successfully");
                        }

                        self.ui_state.reset_pages();
                        self.show_scan_drives_page();
                    }
                }
            });
    }

    fn render_disk_scan_complete_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()))
            .show(ctx, |ui| {
                ui.heading("Scan Complete");

                ui.horizontal(|ui| {
                    if ui.button("Back to Main Menu").clicked() {
                        self.ui_state.change_page_no_history(ShearsPage::MainPage);
                    }

                    if ui.button("Start new scan").clicked() {
                        self.show_scan_drives_page();
                    }
                });

                let mut selected_folder: Option<PathBuf> = None;
                if let Some(folders) = &self.scan_state.scan_results {
                    for folder in folders {
                        if ui.button(folder.to_string_lossy()).clicked() {
                            selected_folder = Some(folder.clone());
                        }
                    }
                } else {
                    // somehow here without the thread result, fallback to main page
                    log::warn!("going back");
                    self.ui_state.reset_pages();
                }

                if let Some(folder) = &selected_folder {
                    self.set_folder(folder);
                }
            });
    }

    fn render_folder_selected_page_header(&mut self, ui: &mut egui::Ui) -> bool {
        ui.horizontal(|ui| {
            let string = if self.ui_state.get_last_page() == ShearsPage::DiskScanComplete {
                "Select another version from scan"
            } else {
                "Back"
            };

            if ui.button(string).clicked() {
                self.ui_state.go_back();
            }
        });

        if let Some(siege_path) = &mut self.folder_state.siege_path {
            ui.heading(
                egui::RichText::new(siege_path.display().to_string())
                    .size(15.0)
                    .monospace(),
            );
        }

        if !self.folder_state.features_availability.has_forge_files {
            ui.label(egui::RichText::new("Folder does not contain FORGE files. Make sure you selected the correct folder.").color(egui::Color32::LIGHT_RED));
            return false;
        }

        true
    }

    fn validate_ui_state_checkbox(&mut self, level: ForgeTextureQualityLevel) {
        if level == ForgeTextureQualityLevel::Low {
            // this function will never be called with the low textures option
            return;
        }

        let level_idx = level as usize;
        let checkboxes = &mut self.ui_state.checkbox_textures;

        let ultra_idx = ForgeTextureQualityLevel::Ultra as usize;
        if level_idx < ultra_idx {
            checkboxes
                .get_mut((level_idx + 1)..=ultra_idx)
                .expect("Out of bounds error")
                .fill(false);
        }

        if *checkboxes.get(level_idx).expect("Out of bounds error") {
            let medium_idx = ForgeTextureQualityLevel::Medium as usize;
            checkboxes
                .get_mut(medium_idx..level_idx)
                .expect("Out of bounds error")
                .fill(true);
        }
    }

    fn render_folder_selected_page_available_features(&mut self, ui: &mut egui::Ui) {
        // the texture checkboxes make sure you cant select for example, high without low, and so on up to ultra
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("Features"));
                if ui.button("Refresh").clicked() {
                    self.refresh_feature_availablity();
                }
            });
            ui.label("Choose what you want to keep");

            ui.separator();

            for quality_level in
                ForgeTextureQualityLevel::Low as usize..=ForgeTextureQualityLevel::Ultra as usize
            {
                let forge_texture_quality_level =
                    ForgeTextureQualityLevel::convert_from_i32(quality_level as i32)
                        .expect("Failed to convert i32 to ForgeTextureQualityLevel");

                ui.add_enabled_ui(
                    self.folder_state
                        .features_availability
                        .get_texture(quality_level)
                        .0,
                    |ui| {
                        if ui
                            .checkbox(
                                self.ui_state
                                    .checkbox_textures
                                    .get_mut(quality_level)
                                    .expect("Out of bounds error"),
                                format!(
                                    "{} Textures ({})",
                                    forge_texture_quality_level,
                                    humansize::format_size(
                                        self.folder_state
                                            .features_availability
                                            .get_texture(quality_level)
                                            .1,
                                        humansize::WINDOWS
                                    )
                                ),
                            )
                            .clicked()
                        {
                            self.validate_ui_state_checkbox(forge_texture_quality_level);
                            self.compute_possible_space_freed();
                        }
                    },
                );
            }

            ui.separator();

            ui.add_enabled_ui(self.folder_state.features_availability.videos.0, |ui| {
                if ui
                    .checkbox(
                        &mut self.ui_state.checkbox_videos,
                        format!(
                            "Videos ({})",
                            humansize::format_size(
                                self.folder_state.features_availability.videos.1,
                                humansize::WINDOWS
                            )
                        ),
                    )
                    .clicked()
                {
                    self.compute_possible_space_freed();
                }
            });

            if self
                .persistent_settings_storage
                .enable_experimental_features
            {
                ui.add_enabled_ui(self.folder_state.features_availability.events.0, |ui| {
                    if ui
                        .checkbox(
                            &mut self.ui_state.checkbox_events,
                            format!(
                                "Event files ({}) [EXPERIMENTAL]",
                                humansize::format_size(
                                    self.folder_state.features_availability.events.1,
                                    humansize::WINDOWS
                                )
                            ),
                        )
                        .clicked()
                    {
                        self.compute_possible_space_freed();
                    }
                });
            }
        });
    }

    fn render_folder_selected_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()))
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.vertical(|ui| {
                        if !self.render_folder_selected_page_header(ui) {
                            return;
                        }

                        self.render_folder_selected_page_available_features(ui);

                        ui.label(format!(
                            "Space saved: {}",
                            humansize::format_size(
                                self.ui_state.label_possible_space_saved,
                                humansize::WINDOWS
                            )
                        ));

                        if ui.button("Shear!").clicked()
                            && native_dialog::DialogBuilder::message()
                            .set_level(native_dialog::MessageLevel::Warning)
                            .set_title("Read before proceeding")
                            .set_text("Are you sure you want to continue? This change is permanent and cannot be undone. After proceeding you must verify your installation and re-download any affected files.")
                            .confirm()
                            .show()
                            .expect("Failed to show dialog")
                        {
                            if is_siege_running(&mut self.system_information) {
                                native_dialog::DialogBuilder::message()
                                    .set_level(native_dialog::MessageLevel::Error)
                                    .set_title("Error")
                                    .set_text("Rainbow Six Siege is currently running! Please close it before shearing.")
                                    .alert()
                                    .show()
                                    .expect("Failed to show dialog");
                                return;
                            }
                            if self.execute_shearing() {
                                let message = self.folder_state.siege_path.as_ref()
                                    .map(|path| format!("\"{}\" has been successfully sheared.", path.display()))
                                    .unwrap_or_else(|| "The Siege folder has been successfully sheared.".to_owned());

                                native_dialog::DialogBuilder::message()
                                    .set_level(native_dialog::MessageLevel::Info)
                                    .set_title("Success")
                                    .set_text(&message)
                                    .alert()
                                    .show()
                                    .expect("Failed to show dialog");
                            } else {
                                native_dialog::DialogBuilder::message()
                                    .set_level(native_dialog::MessageLevel::Error)
                                    .set_title("Failure")
                                    .set_text("Shearing failed.")
                                    .alert()
                                    .show()
                                    .expect("Failed to show dialog");
                            }
                        }
                    }
                );
            });
        });
    }

    fn render_modals(&mut self, ctx: &egui::Context) -> bool {
        if self.ui_state.get_modal(ShearsModals::About as usize) {
            let modal = egui::Modal::new(egui::Id::new("ModalAbout")).show(ctx, |ui| {
                ui.set_width(350.0);

                ui.heading("Shears");
                ui.vertical(|ui| {
                    ui.label(format!("Version: {} ", env!("CARGO_PKG_VERSION")));
                    ui.label(format!("Built using {}", env!("RUSTC_VERSION")));

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("Made by ");
                        ui.hyperlink_to("Lungu", "https://github.com/lungu19");
                    });

                    ui.hyperlink_to("Source code", "https://github.com/lungu19/shears");

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("Join the ");
                        ui.hyperlink_to("R6 Throwback", "https://discord.gg/yxpT6EChgr");
                        ui.label(" Discord server");
                    });
                });

                if ui.button("Close").clicked() {
                    ui.close();
                }
            });

            if modal.should_close() {
                *self.ui_state.get_modal_mut(ShearsModals::About as usize) = false;
            }
        }

        if self.ui_state.get_modal(ShearsModals::Settings as usize) {
            let modal = egui::Modal::new(egui::Id::new("ModalSettings")).show(ctx, |ui| {
                ui.set_width(400.0);

                ui.heading("Settings");
                ui.vertical(|ui| {
                    ui.checkbox(
                        &mut self
                            .persistent_settings_storage
                            .enable_shears_update_check_on_startup,
                        "Automatically check for updates on app startup",
                    );

                    ui.checkbox(
                        &mut self.persistent_settings_storage.use_loose_selection,
                        "Enable loose selection of the Siege folder",
                    );

                    ui.checkbox(
                        &mut self
                            .persistent_settings_storage
                            .enable_experimental_features,
                        "Enable experimental features",
                    );
                });

                if ui.button("Close").clicked() {
                    self.persistent_settings_storage.save_to_file();
                    ui.close();
                }
            });

            if modal.should_close() {
                *self.ui_state.get_modal_mut(ShearsModals::Settings as usize) = false;
            }
        }

        let mut any_modal_open = false;
        for modal in ShearsModals::START as usize..=ShearsModals::END as usize {
            any_modal_open &= self.ui_state.get_modal(modal);
        }
        any_modal_open
    }

    fn render_drag_and_drop_preview(&mut self, ctx: &egui::Context) {
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let text = ctx.input(|i| {
                if i.raw.hovered_files.len() > 1 {
                    return "Only drop a single file.".to_owned();
                }

                if let Some(file) = i.raw.hovered_files.first()
                    && let Some(path) = &file.path
                {
                    let is_valid_exe = path.is_file()
                        && path
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));

                    if is_valid_exe {
                        return format!("Drop to select executable:\n{}", path.display());
                    } else {
                        return "Only .exe files are accepted.".to_owned();
                    }
                }

                "You shouldn't be able to see this message.".to_owned() // Fallback
            });

            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("file_drop_target"),
            ));

            let content_rect = ctx.content_rect();
            painter.rect_filled(content_rect, 0.0, egui::Color32::from_black_alpha(192));
            painter.text(
                content_rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::TextStyle::Body.resolve(&ctx.style()),
                egui::Color32::WHITE,
            );
        }

        ctx.input(|i| {
            if i.raw.dropped_files.len() == 1
                && let Some(path) = &i
                    .raw
                    .dropped_files
                    .first()
                    .expect("Out of bounds error")
                    .path
            {
                // Check if the dropped path is specifically a file (ignores directories)
                if path.is_file()
                    && path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
                {
                    // Get the parent folder of that file
                    if let Some(parent) = path.parent() {
                        self.set_folder(parent);
                    }
                }
            }
        });
    }

    fn render_drag_and_drop_preview_loose(&mut self, ctx: &egui::Context) {
        // ui
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let text = ctx.input(|i| {
                if i.raw.hovered_files.len() > 1 {
                    return "Please drop a single file or folder.".to_owned();
                }

                if let Some(file) = i.raw.hovered_files.first()
                    && let Some(path) = &file.path
                {
                    let target_path = if path.is_dir() {
                        path.clone()
                    } else {
                        path.parent()
                            .map_or_else(|| path.clone(), |p| p.to_path_buf()) // If it's a file, copy the parent folder
                    };
                    return format!("Drop to select folder:\n{}", target_path.display());
                }

                "Drop to select folder".to_owned() // Fallback
            });

            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("file_drop_target"),
            ));

            let content_rect = ctx.content_rect();
            painter.rect_filled(content_rect, 0.0, egui::Color32::from_black_alpha(192));
            painter.text(
                content_rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::TextStyle::Body.resolve(&ctx.style()),
                egui::Color32::WHITE,
            );
        }

        // logic
        ctx.input(|i| {
            if i.raw.dropped_files.len() == 1
                && let Some(path) = &i
                    .raw
                    .dropped_files
                    .first()
                    .expect("Out of bounds error")
                    .path
            {
                // If a file is dropped instead of a folder, use its parent folder path instead
                let folder_path = if path.is_dir() {
                    Some(path.clone())
                } else {
                    path.parent().map(|p| p.to_path_buf())
                };

                if let Some(p) = folder_path {
                    self.set_folder(&p);
                }
            }
        });
    }
}

impl eframe::App for ShearsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_menu_bar(ctx);
        self.render_main_content(ctx);

        // if any modal is being rendered, disable drag and drop behaviour
        if !self.render_modals(ctx) {
            if self.persistent_settings_storage.use_loose_selection {
                self.render_drag_and_drop_preview_loose(ctx);
            } else {
                self.render_drag_and_drop_preview(ctx);
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.persistent_settings_storage.save_to_file();
    }
}
