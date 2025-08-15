use std::path::Path;

use egui::Color32;

use crate::{
    helpers::{
        delete_texture_files, delete_videos_folder, get_shearing_features_availability,
        windows_confirmation_dialog, write_streaminginstall,
    },
    types::{ForgeTextureQualityLevel, ShearsFolderState, ShearsModals, ShearsPage, ShearsUiState},
};

#[derive(Default)]
pub struct ShearsApp {
    folder_state: ShearsFolderState,
    ui_state: ShearsUiState,
}

impl ShearsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        catppuccin_egui::set_theme(&cc.egui_ctx, catppuccin_egui::MACCHIATO);
        Default::default()
    }

    pub fn set_folder(&mut self, folder: &Path) {
        self.folder_state.siege_path = Some(folder.to_path_buf());
        self.refresh_feature_availablity();
    }

    pub fn refresh_feature_availablity(&mut self) {
        let siege_path = self
            .folder_state
            .siege_path
            .as_ref()
            .expect("Failed to get folder_state.siege_path");
        self.folder_state.features_availability = get_shearing_features_availability(siege_path);

        self.ui_state.page = ShearsPage::FolderSelected;

        // set the feature checkboxes accordingly
        for quality_level in
            ForgeTextureQualityLevel::Low as usize..=ForgeTextureQualityLevel::Ultra as usize
        {
            self.ui_state.checkbox_textures[quality_level] =
                self.folder_state.features_availability.textures[quality_level].0;
        }

        self.folder_state.features_availability.textures[ForgeTextureQualityLevel::Low as usize]
            .0 = false; // let's avoid users making their games completely unplayable

        self.ui_state.checkbox_videos = self.folder_state.features_availability.videos.0;

        self.compute_possible_space_freed();
    }

    fn compute_possible_space_freed(&mut self) {
        self.ui_state.label_possible_space_saved = 0;

        for i in
            ForgeTextureQualityLevel::Medium as usize..=ForgeTextureQualityLevel::Ultra as usize
        // the user is not able to remove low textures so we can just skip it here
        {
            if !self.ui_state.checkbox_textures[i] {
                self.ui_state.label_possible_space_saved +=
                    self.folder_state.features_availability.textures[i].1;
            }
        }

        if !self.ui_state.checkbox_videos {
            self.ui_state.label_possible_space_saved +=
                self.folder_state.features_availability.videos.1;
        }
    }

    fn execute_shearing(&mut self) {
        let mut min_texture_quality_level = ForgeTextureQualityLevel::Low;
        for i in (ForgeTextureQualityLevel::Medium.convert_to_i32()
            ..=ForgeTextureQualityLevel::Ultra.convert_to_i32())
            .rev()
        {
            let level = ForgeTextureQualityLevel::convert_from_i32(i)
                .expect("Failed to convert i32 to ForgeTextureQualityLevel");
            if self.ui_state.checkbox_textures[level as usize] {
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

            write_streaminginstall(path).expect("Failed to write streaming install");

            self.set_folder(path);
        }
    }
}

impl ShearsApp {
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
                        self.ui_state.modals[ShearsModals::About as usize] = true;
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
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(75.0))
            .show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    if ui
                        .button("Drag and drop or click this to select the Siege folder...")
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.set_folder(&path);
                        }
                    }
                });
            });
    }

    fn render_folder_selected_page_header(&mut self, ui: &mut egui::Ui) -> bool {
        ui.horizontal(|ui| {
            if ui.button("Select another folder").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.set_folder(&path);
                }
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
            ui.label(egui::RichText::new("Folder does not contain FORGE files. Make sure you selected the correct folder.").color(Color32::LIGHT_RED));
            return false;
        }

        true
    }

    fn validate_ui_state_checkbox(&mut self, level: ForgeTextureQualityLevel) {
        if level == ForgeTextureQualityLevel::Low {
            // this function will never be called with the low textures option
            self.compute_possible_space_freed();
            return;
        }

        let level_idx = level as usize;
        let checkboxes = &mut self.ui_state.checkbox_textures;

        let ultra_idx = ForgeTextureQualityLevel::Ultra as usize;
        if level_idx < ultra_idx {
            checkboxes[(level_idx + 1)..=ultra_idx].fill(false);
        }

        if checkboxes[level_idx] {
            let medium_idx = ForgeTextureQualityLevel::Medium as usize;
            checkboxes[medium_idx..level_idx].fill(true);
        }

        self.compute_possible_space_freed();
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
                    self.folder_state.features_availability.textures[quality_level].0,
                    |ui| {
                        if ui
                            .checkbox(
                                &mut self.ui_state.checkbox_textures[quality_level],
                                format!(
                                    "{} Textures ({})",
                                    forge_texture_quality_level,
                                    humansize::format_size(
                                        self.folder_state.features_availability.textures
                                            [quality_level]
                                            .1,
                                        humansize::WINDOWS
                                    )
                                ),
                            )
                            .clicked()
                        {
                            self.validate_ui_state_checkbox(forge_texture_quality_level);
                        }
                    },
                );
            }

            ui.separator();

            ui.add_enabled_ui(self.folder_state.features_availability.videos.0, |ui| {
                ui.checkbox(
                    &mut self.ui_state.checkbox_videos,
                    format!(
                        "Videos ({})",
                        humansize::format_size(
                            self.folder_state.features_availability.videos.1,
                            humansize::WINDOWS
                        )
                    ),
                );
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
                            self.execute_shearing();
                        }
                    });
                });
            });
    }

    fn render_modals(&mut self, ctx: &egui::Context) -> bool {
        if self.ui_state.modals[ShearsModals::About as usize] {
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
                        ui.hyperlink_to("R6 Throwback", "https://discord.gg/JGA9WPF4K8");
                        ui.label(" Discord server");
                    });
                });

                if ui.button("Close").clicked() {
                    ui.close();
                }
            });

            if modal.should_close() {
                self.ui_state.modals[ShearsModals::About as usize] = false;
            }
        }

        self.ui_state.modals[ShearsModals::About as usize] // if other modals are added, use an OR operator to see if any modal is open
    }

    fn render_drag_and_drop_preview(&mut self, ctx: &egui::Context) {
        // ui
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let text = ctx.input(|i| {
                if i.raw.hovered_files.len() > 1 {
                    return "Please drop a single file or folder.".to_owned();
                }

                if let Some(file) = i.raw.hovered_files.first() {
                    if let Some(path) = &file.path {
                        let target_path = if path.is_dir() {
                            path.clone()
                        } else {
                            path.parent()
                                .map_or_else(|| path.clone(), |p| p.to_path_buf()) // If it's a file, copy the parent folder
                        };
                        return format!("Drop to select folder:\n{}", target_path.display());
                    }
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
                Color32::WHITE,
            );
        }

        // logic
        ctx.input(|i| {
            if i.raw.dropped_files.len() == 1 {
                if let Some(path) = &i.raw.dropped_files[0].path {
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
