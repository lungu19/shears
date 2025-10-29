use crate::{
    helpers::{
        delete_events_folder, delete_texture_files, delete_videos_folder,
        get_shearing_features_availability, is_siege_running, windows_confirmation_dialog,
        windows_error_dialog, windows_information_dialog, write_streaminginstall,
    },
    types::{ForgeTextureQualityLevel, ShearsFolderState, ShearsModals, ShearsPage, ShearsUiState},
};

#[derive(Default)]
pub struct ShearsApp {
    folder_state: ShearsFolderState,
    ui_state: ShearsUiState,
    system_information: sysinfo::System,
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

        Self {
            system_information: sysinfo::System::new(),
            ..Self::default()
        }
    }

    pub fn set_folder(&mut self, folder: &std::path::Path) {
        self.folder_state.siege_path = Some(folder.to_path_buf());
        self.refresh_feature_availablity();
    }

    pub fn refresh_feature_availablity(&mut self) {
        let siege_path =
            self.folder_state.siege_path.as_ref().expect(
                "ShearsApp.refresh_feature_availablity: Failed to get folder_state.siege_path",
            );
        self.folder_state.features_availability = get_shearing_features_availability(siege_path);

        self.ui_state.page = ShearsPage::FolderSelected;

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
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        *self.ui_state.get_modal_mut(ShearsModals::About as usize) = true;
                    }
                });

                #[cfg(debug_assertions)]
                ui.label(
                    egui::RichText::new("Debug build")
                        .small()
                        .color(ui.visuals().warn_fg_color),
                )
                .on_hover_text("compiled with debug assertions enabled.");
            });
        });
    }

    fn render_main_content(&mut self, ctx: &egui::Context) {
        match self.ui_state.page {
            ShearsPage::SelectFolder => self.render_select_folder_page(ctx),
            ShearsPage::FolderSelected => self.render_folder_selected_page(ctx),
        }
    }

    fn render_select_folder_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()))
            .show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.style_mut().visuals.button_frame = false;
                    if ui
                        .button("Drag and drop or click to select the Siege folder...")
                        .clicked()
                        && let Some(path) = rfd::FileDialog::new().pick_folder()
                    {
                        self.set_folder(&path);
                    }
                });
            });
    }

    fn render_folder_selected_page_header(&mut self, ui: &mut egui::Ui) -> bool {
        ui.horizontal(|ui| {
            if ui.button("Select another folder").clicked()
                && let Some(path) = rfd::FileDialog::new().pick_folder()
            {
                self.set_folder(&path);
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
                            && windows_confirmation_dialog(
                                "Warning",
                                "Are you sure you want to continue? This change is permanent and cannot be undone. After proceeding you must verify your installation and re-download any affected files.",
                            )
                        {
                            if is_siege_running(&mut self.system_information) {
                                windows_error_dialog("Error", "Rainbow Six Siege is currently running! Please close it before shearing.");
                                return;
                            }
                            if self.execute_shearing() {
                                let message = self.folder_state.siege_path.as_ref()
                                    .map(|path| format!("{} has been successfully sheared.", path.display()))
                                    .unwrap_or_else(|| "The Siege folder has been successfully sheared.".to_owned());
                                windows_information_dialog("Success", &message);
                            } else {
                                windows_error_dialog("Failure", "Shearing failed.");
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

        let mut any_modal_open = false;
        for modal in ShearsModals::START as usize..=ShearsModals::END as usize {
            any_modal_open &= self.ui_state.get_modal(modal);
        }
        any_modal_open
    }

    fn render_drag_and_drop_preview(&mut self, ctx: &egui::Context) {
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

            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
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
            self.render_drag_and_drop_preview(ctx);
        }
    }
}
