use crossbeam_channel::Sender;
use eframe::{
    egui::{self, Context},
    epaint::Vec2,
};
use egui_file::FileDialog;
use egui_notify::Toasts;
use std::{path::PathBuf, time::Duration};

use crate::{
    gui::modals,
    manager::{custom_effect::CustomEffect, profile::Profile},
    DENY_HIDING,
};

use super::{GuiMessage, LoadedEffect};

pub struct MenuBarState {
    gui_sender: Sender<GuiMessage>,
    load_profile_dialog: FileDialog,
    load_effect_dialog: FileDialog,
    save_profile_dialog: FileDialog,
}

impl MenuBarState {
    pub(super) fn new(gui_sender: Sender<GuiMessage>) -> Self {
        Self {
            gui_sender,
            load_profile_dialog: FileDialog::open_file(None).default_size(Vec2::splat(300.0)),
            load_effect_dialog: FileDialog::open_file(None).default_size(Vec2::splat(300.0)),
            save_profile_dialog: FileDialog::save_file(None).default_size(Vec2::splat(300.0)),
        }
    }

    pub fn show(&mut self, ctx: &Context, ui: &mut egui::Ui, current_profile: &mut Profile, current_effect: &mut LoadedEffect, changed: &mut bool, toasts: &mut Toasts, app_settings: &mut crate::settings::Settings) {
        self.show_menu(ctx, ui, toasts, app_settings, current_profile, changed);
        self.handle_load_profile(ctx, current_profile, changed, toasts);
        self.handle_save_profile(ctx, current_profile, toasts);
        self.handle_load_effect(ctx, current_effect, changed, toasts);
    }

    fn handle_load_profile(&mut self, ctx: &Context, current_profile: &mut Profile, changed: &mut bool, toasts: &mut Toasts) {
        if self.load_profile_dialog.show(ctx).selected() {
            if let Some(path) = self.load_profile_dialog.path().map(|p| p.to_path_buf()) {
                match Profile::load_profile(&path) {
                    Ok(profile) => {
                        *current_profile = profile;
                        *changed = true;
                    }
                    Err(_) => {
                        toasts.error("Could not load profile.").duration(Some(Duration::from_millis(5000))).closable(true);
                    }
                }
                self.update_paths(path);
            }
        }
    }

    fn handle_save_profile(&mut self, ctx: &Context, current_profile: &mut Profile, toasts: &mut Toasts) {
        if self.save_profile_dialog.show(ctx).selected() {
            if let Some(path) = self.save_profile_dialog.path().map(|p| p.to_path_buf()) {
                if current_profile.save_profile(&path).is_err() {
                    toasts.error("Could not save profile.").duration(Some(Duration::from_millis(5000))).closable(true);
                }
                self.update_paths(path);
            }
        }
    }

    fn handle_load_effect(&mut self, ctx: &Context, current_effect: &mut LoadedEffect, changed: &mut bool, toasts: &mut Toasts) {
        if self.load_effect_dialog.show(ctx).selected() {
            if let Some(path) = self.load_effect_dialog.path().map(|p| p.to_path_buf()) {
                match CustomEffect::from_file(&path) {
                    Ok(effect) => {
                        *current_effect = LoadedEffect::queued(effect);
                        *changed = true;
                    }
                    Err(_) => {
                        toasts.error("Could not load custom effect.").duration(Some(Duration::from_millis(5000))).closable(true);
                    }
                }
                self.update_paths(path);
            }
        }
    }

    fn update_paths(&mut self, path: PathBuf) {
        let mut save_paths = |path: PathBuf| {
            self.load_profile_dialog.set_path(path.clone());
            self.load_effect_dialog.set_path(path.clone());
            self.save_profile_dialog.set_path(path);
        };

        if path.exists() {
            if path.is_file() {
                if let Some(parent) = path.parent() {
                    save_paths(parent.to_path_buf())
                }
            } else {
                save_paths(path)
            }
        }
    }

    #[allow(unused_variables)]
    fn show_menu(&mut self, ctx: &Context, ui: &mut egui::Ui, toasts: &mut Toasts, app_settings: &mut crate::settings::Settings, current_profile: &mut Profile, changed: &mut bool) {
        use egui::menu;
        use eframe::epaint::Color32;
        use eframe::egui::RichText;

        menu::bar(ui, |ui| {
            ui.menu_button("Profile", |ui| {
                if ui.button("Open").clicked() {
                    self.load_profile_dialog.open();
                }
                if ui.button("Save").clicked() {
                    self.save_profile_dialog.open();
                }
            });

            ui.menu_button("Effect", |ui| {
                if ui.button("Open").clicked() {
                    self.load_effect_dialog.open();
                }
            });
            
            ui.menu_button("Customise", |ui| {
                let mut save_needed = false;
                
                #[cfg(target_os = "windows")]
                {
                    if let Ok(current_exe) = std::env::current_exe() {
                        if let Ok(auto) = auto_launch::AutoLaunchBuilder::new()
                            .set_app_name("LegionKeyboardRGB")
                            .set_app_path(&current_exe.to_string_lossy())
                            .build()
                        {
                            let mut auto_launch_toggled = app_settings.start_with_windows;
                            if ui.checkbox(&mut auto_launch_toggled, "Start with Windows").changed() {
                                app_settings.start_with_windows = auto_launch_toggled;
                                save_needed = true;
                                if auto_launch_toggled {
                                    let _ = auto.enable();
                                } else {
                                    let _ = auto.disable();
                                }
                            }
                        }
                    }
                    
                    if ui.button("Use Windows Accent Color").clicked() {
                        use winreg::enums::*;
                        use winreg::RegKey;
                        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                        if let Ok(dwm) = hkcu.open_subkey("Software\\Microsoft\\Windows\\DWM") {
                            if let Ok(color) = dwm.get_value::<u32, _>("ColorizationColor") {
                                let r = ((color >> 16) & 0xFF) as u8;
                                let g = ((color >> 8) & 0xFF) as u8;
                                let b = (color & 0xFF) as u8;
                                
                                current_profile.rgb_zones = crate::manager::profile::arr_to_zones([
                                    r, g, b, r, g, b, r, g, b, r, g, b
                                ]);
                                *changed = true;
                                toasts.success("Applied Windows Accent Color!").duration(Some(Duration::from_millis(3000))).closable(true);
                            } else {
                                toasts.error("Could not read accent color.").duration(Some(Duration::from_millis(3000))).closable(true);
                            }
                        } else {
                            toasts.error("Could not access DWM registry key.").duration(Some(Duration::from_millis(3000))).closable(true);
                        }
                    }
                }
                
                if ui.checkbox(&mut app_settings.start_minimized, "Start Minimized").changed() {
                    save_needed = true;
                }
                
                if save_needed {
                    app_settings.save();
                }
            });

            let about_modal = modals::about(ctx);
            if ui.button("About").clicked() {
                about_modal.open();
            }
            
            if ui.button(RichText::new("Reset").color(Color32::RED)).clicked() {
                crate::settings::Settings::delete();
                *app_settings = crate::settings::Settings::default();
                *current_profile = Profile::default();
                *changed = true;
                toasts.success("Settings and colors reset to default!").duration(Some(Duration::from_millis(3000))).closable(true);
            }

            if !*DENY_HIDING && ui.button("Exit").clicked() {
                self.gui_sender.send(GuiMessage::Quit).unwrap();
            }

            #[cfg(target_os = "windows")]
            {
                use crate::console;
                use eframe::{egui::Layout, emath::Align};
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("📜").clicked() {
                        if !console::alloc_with_color_support() {
                            toasts.error("Could not allocate debug terminal.").duration(Some(Duration::from_millis(5000))).closable(true);
                        }
                        println!("Debug terminal enabled.");
                    }
                });
            }
        });
    }
}
