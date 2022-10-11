use super::camera;
use crate::ui::state::AppState;
use eframe::egui::{CentralPanel, Context, Direction, Layout};

pub fn show(ctx: &Context, state: &mut AppState) {
  if state.camera.is_some() {
    if state.panes.camera_media {
      camera::media::show(ctx, state);
    }

    if state.panes.camera_info {
      camera::info::show(ctx, state);
    }

    if state.panes.camera_settings {
      camera::settings::show(ctx, state);
    }

    camera::central_view::show(ctx, state);
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
