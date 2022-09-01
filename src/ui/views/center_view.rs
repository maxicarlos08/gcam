use crate::{app::AppState, error::CatchAppResult};
use eframe::{
    egui::{Button, CentralPanel, Context, Direction, Frame, Layout},
    emath::Align,
};
use gcam_lib::utils::geom::fit_size_into;

pub fn show(ctx: &Context, state: &mut AppState) {
    let camera = state.camera.as_mut().unwrap();
    let mut set_live_view = None;

    CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        camera.info.abilities.camera_operations().capture_preview(),
                        Button::new(if camera.live_view_enabled {
                            "Stop live view"
                        } else {
                            "Start live view"
                        }),
                    )
                    .clicked()
                {
                    set_live_view = Some(!camera.live_view_enabled);
                }
            });

            ui.with_layout(
                Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    Frame::dark_canvas(ui.style()).show(ui, |ui| {
                        if let Some(preview) = &state.last_preview_capture {
                            ui.image(
                                preview,
                                fit_size_into(preview.size_vec2(), ui.available_size()),
                            );
                        } else {
                            ui.label("No preview has been captured");
                        }
                    });
                },
            );
        })
    });

    if let Some(live_view_enabled) = set_live_view {
        state.set_live_view(live_view_enabled).catch(state);
    }
}
