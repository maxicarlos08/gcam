use crate::app::{AppState, UICamera};
use camera_controller::messages::CameraStatus;
use eframe::{
    egui::{Context, Frame, Grid, ScrollArea, SidePanel, Ui, Window},
    emath::Align2,
    epaint::{Color32, Vec2},
};
use gphoto2::{
    abilities::CameraDriverStatus,
    filesys::{AccessType, FilesystemType, StorageInfo, StorageType},
};

pub fn show(ctx: &Context, state: &mut AppState) {
    if let Some(camera) = &state.camera {
        additional_info_window(ctx, &mut state.open_dialogs.camera_info_text, camera);

        SidePanel::left("camera_info_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Camera information");
                ui.separator();
                ScrollArea::vertical().show(ui, |ui| {
                    ui.collapsing("Driver Information", |ui| {
                        driver_information(ui, &mut state.open_dialogs.camera_info_text, camera)
                    });

                    ui.collapsing("Camera Status", |ui| camera_status(ui, &camera.info.status));

                    ui.collapsing("Camera Storages", |ui| {
                        for (i, storage) in camera.info.storages.iter().enumerate() {
                            storage_display(ui, storage, i);
                        }
                    })
                })
            });
    }
}

#[inline]
fn camera_status(ui: &mut Ui, status: &CameraStatus) {
    ScrollArea::horizontal().show(ui, |ui| {
        Grid::new("camera_status_grid")
            .striped(true)
            .show(ui, |ui| {
                if let Some(manufacturer) = &status.manufacturer {
                    ui.label("Manufacturer");
                    ui.label(manufacturer);
                    ui.end_row();
                }

                if let Some(serial_number) = &status.serial_number {
                    ui.label("Serial number");
                    ui.label(serial_number);
                    ui.end_row();
                }

                if let Some(model) = &status.model {
                    ui.label("Model");
                    ui.label(model);
                    ui.end_row();
                }

                if let Some(ac_powered) = status.ac_power {
                    ui.label("AC powered");
                    ui.label(if ac_powered { "Yes" } else { "No" });
                    ui.end_row();
                }

                if let Some(battery_level) = status.battery_level {
                    ui.label("Battery level");
                    ui.label(format!("{}%", battery_level * 100f32));
                    ui.end_row();
                }
            })
    });
}

#[inline]
fn storage_display(ui: &mut Ui, storage: &StorageInfo, index: usize) {
    ui.collapsing(format!("Storage #{}", index), |ui| {
        ScrollArea::horizontal().show(ui, |ui| {
            Grid::new("camera_storages_grid")
                .striped(true)
                .show(ui, |ui| {
                    if let Some(label) = storage.label() {
                        ui.label("Label");
                        ui.label(label);
                        ui.end_row();
                    }

                    if let Some(description) = storage.description() {
                        ui.label("Description");
                        ui.label(description);
                        ui.end_row();
                    }

                    if let Some(storage_type) = storage.storage_type() {
                        ui.label("Storage type");
                        ui.label(match storage_type {
                            StorageType::Unknown => "Unknown",
                            StorageType::FixedRam => "Fixed RAM (eg. SDRAM)",
                            StorageType::FixedRom => "Fixed ROM",
                            StorageType::RemovableRam => "Removable RAM (SD cards)",
                            StorageType::RemovableRom => "Removable ROM",
                        });
                        ui.end_row();
                    }

                    if let Some(filesystem_type) = storage.filesystem_type() {
                        ui.label("Filesystem type");
                        ui.label(match filesystem_type {
                            FilesystemType::Dcf => "DCIM filesystem",
                            FilesystemType::Flat => "Flat filesystem",
                            FilesystemType::Tree => "Tree filesystem",
                            FilesystemType::Unknown => "Unknown filesystem",
                        });
                        ui.end_row();
                    }

                    if let Some(access) = storage.access_type() {
                        ui.label("Access permissions");
                        ui.label(match access {
                            AccessType::Ro => "Read only",
                            AccessType::RoDelete => "Read only & delete",
                            AccessType::Rw => "Read write",
                        });
                        ui.end_row();
                    }

                    if let Some(capacity) = storage.capacity() {
                        ui.label("Capacity");
                        ui.label(format!("{} Kb", capacity));
                        ui.end_row();
                    }

                    if let Some(free) = storage.free() {
                        ui.label("Free");
                        ui.label(format!("{} Kb", free));
                        ui.end_row();
                    }
                })
        });
    });
}

#[inline]
fn driver_information(ui: &mut Ui, additional_dialog_open: &mut bool, camera: &UICamera) {
    Grid::new("camera_driver_info_grid")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Model");
            ui.label(&camera.info.model);
            ui.end_row();

            ui.label("Port");
            ui.label(&camera.info.port);
            ui.end_row();

            ui.label("Driver status");
            ui.label(match camera.info.abilities.driver_status() {
                CameraDriverStatus::Production => "Stable",
                CameraDriverStatus::Testing => "Testing",
                CameraDriverStatus::Experimental => "Experimental",
                CameraDriverStatus::Deprecated => "Deprecated",
            });

            ui.end_row();
        });

    if ui.button("Additional information").clicked() {
        *additional_dialog_open = true;
    }
}

#[inline]
fn additional_info_window(ctx: &Context, open: &mut bool, camera: &UICamera) {
    Window::new("Additional camera info")
        .collapsible(false)
        .resizable(true)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .show(ctx, |ui| {
            if let Some(summary) = &camera.info.summary {
                ui.collapsing("Summary", |ui| ui.monospace(summary));
            }

            if let Some(manual) = &camera.info.manual {
                ui.collapsing("Manual", |ui| ui.monospace(manual));
            }

            if let Some(about) = &camera.info.about {
                ui.collapsing("About", |ui| ui.monospace(about));
            }
        });
}
