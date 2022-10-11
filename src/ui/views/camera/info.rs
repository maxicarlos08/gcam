use crate::{
  cam_thread::settings::{display_radio_choice, CameraSettings, StaticWidget},
  ui::state::{camera::UICamera, AppState},
};
use eframe::{
  egui::{Context, Grid, ScrollArea, SidePanel, Ui, Window},
  emath::Align2,
  epaint::Vec2,
};
use gphoto2::{
  abilities::CameraDriverStatus,
  filesys::{AccessType, FilesystemType, StorageInfo, StorageType},
};

pub fn show(ctx: &Context, state: &mut AppState) {
  if let Some(camera) = &state.camera {
    additional_info_window(ctx, &mut state.open_dialogs.camera_info_text, camera);

    SidePanel::left("camera_info_panel").resizable(true).show(ctx, |ui| {
      ui.heading("Camera information");
      ui.separator();
      ScrollArea::vertical().show(ui, |ui| {
        ui.collapsing("Driver Information", |ui| {
          driver_information(ui, &mut state.open_dialogs.camera_info_text, camera)
        });

        if let Some(StaticWidget::Group { children, .. }) = camera
          .settings
          .as_ref()
          .and_then(|settings| settings.get_child("status").map(|w| &w.widget))
        {
          ui.collapsing("Camera Status", |ui| {
            camera_status(
              ui,
              children.iter().filter_map(|(_, child)| {
                if state.settings.dev_settings.exclude_settings.contains(&child.name) {
                  None
                } else {
                  Some(child)
                }
              }),
            )
          });
        }

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
fn camera_status<'a>(ui: &mut Ui, children: impl Iterator<Item = &'a CameraSettings>) {
  ScrollArea::horizontal().show(ui, |ui| {
    Grid::new("camera_status_grid").striped(true).show(ui, |ui| {
      for child in children {
        ui.label(&child.label);

        match &child.widget {
          StaticWidget::Date { timestamp } => ui.label(timestamp.to_string()),
          StaticWidget::Radio { choice, choices } => {
            ui.label(display_radio_choice(choices, choice))
          }
          StaticWidget::Range { value, .. } => ui.label(value.to_string()), // TODO: Add a progress or an inactive slider bar here
          StaticWidget::Text(text) => ui.label(text),
          StaticWidget::Toggle { undefined, value } => ui.label(match (undefined, value) {
            (false, true) => "Yes",
            (false, false) => "No",
            (true, _) => "Unknown",
          }),
          StaticWidget::Button => {
            todo!("Buttons are not yet implemented");
          }
          StaticWidget::Group { .. } => unreachable!(),
        };

        ui.end_row();
      }
    })
  });
}

#[inline]
fn storage_display(ui: &mut Ui, storage: &StorageInfo, index: usize) {
  ui.collapsing(format!("Storage #{}", index), |ui| {
    ScrollArea::horizontal().show(ui, |ui| {
      Grid::new("camera_storages_grid").striped(true).show(ui, |ui| {
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

        if let Some(capacity) = storage.capacity_kb() {
          ui.label("Capacity");
          ui.label(format!("{} Kb", capacity));
          ui.end_row();
        }

        if let Some(free) = storage.free_kb() {
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
  Grid::new("camera_driver_info_grid").striped(true).show(ui, |ui| {
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
