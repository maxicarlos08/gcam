use crate::{
    error::ToUIError,
    ui::{components, state::AppState, views, windows},
};
use eframe::egui::Context;
use std::time::Duration;

const REPAINT_INTERVAL: Duration = Duration::from_millis(100);

impl eframe::App for AppState {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        if let Err(err) = self.process_events_from_camera_thread(ctx) {
            self.errors.push(err.to_ui_error())
        }

        components::top_menu::show(ctx, frame, self);
        components::bottom_bar::show(ctx, frame);
        views::main::show(ctx, self);

        if self.open_dialogs.settings {
            windows::settings::show(ctx, self);
        }

        {
            let mut pop_error = false;

            for error in &self.errors {
                if windows::error::show(ctx, error) {
                    pop_error = true;
                }
            }

            if pop_error {
                self.errors.pop();
            }
        }

        ctx.request_repaint_after(REPAINT_INTERVAL);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_camera_thread()
    }
}
