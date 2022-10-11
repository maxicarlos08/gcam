use crate::ui::state::AppState;
use eframe::{
  egui::{Context, Layout, Window},
  emath::{Align, Align2},
};

pub fn show(ctx: &Context, state: &mut AppState) {
  // TODO
  Window::new("Settings")
    .collapsible(false)
    .anchor(Align2::CENTER_CENTER, (0., 0.))
    .resizable(false)
    .open(&mut state.open_dialogs.settings)
    .show(ctx, |ui| {
      ui.with_layout(Layout::left_to_right(Align::Max).with_cross_justify(true), |ui| {
        ui.group(|ui| {
          ui.label("TODO");
        });
      })
    });
}
