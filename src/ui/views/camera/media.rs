use eframe::egui::{Context, TopBottomPanel};

use crate::ui::state::AppState;

pub fn show(ctx: &Context, _state: &mut AppState) {
    // TODO
    TopBottomPanel::bottom("camera_media")
        .min_height(100.)
        .resizable(true)
        .show(ctx, |ui| ui.centered_and_justified(|ui| ui.label("TODO")));
}
