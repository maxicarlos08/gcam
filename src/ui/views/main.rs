use crate::app::AppState;
use eframe::egui::{CentralPanel, Context, Direction, Layout};

pub fn show(ctx: &Context, state: &mut AppState) {
    if state.camera.is_some() {
        if state.panes.camera_info {
            super::camera_info::show(ctx, state);
        }

        if state.panes.camera_settings {
            super::camera_settings::show(ctx, state);
        }

        if state.panes.camera_media {
            super::camera_media::show(ctx, state);
        }
    } else {
        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                ui.heading(if state.current_camera.is_some() {
                    "Loading camera..."
                } else {
                    "No camera connected"
                })
            });
        });
    }
}
