use super::views::camera_info;
use crate::app::AppState;
use eframe::egui::{CentralPanel, Context, Direction, Layout};

pub fn show(ctx: &Context, state: &mut AppState) {
    CentralPanel::default().show(ctx, |ui| {
        if state.camera.is_some() {
            if state.panes.camera_info {
                camera_info::show(ctx, ui, state);
            }
        } else {
            ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                ui.heading(if state.current_camera.is_some() {
                    "Loading camera..."
                } else {
                    "No camera connected"
                })
            });
        }
    });
}
