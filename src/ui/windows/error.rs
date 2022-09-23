use crate::error::UiError;
use eframe::{
    egui::{Context, Id, Layout, Window},
    emath::{Align, Align2},
};

pub fn show(ctx: &Context, error: &UiError) -> bool {
    let mut remove = false;

    Window::new(format!("Error: {}", error.title))
        .id(Id::new(error))
        .anchor(Align2::CENTER_CENTER, (0., 0.))
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label(&error.message);
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    remove = true;
                }
            })
        });

    remove
}
