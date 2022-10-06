use crate::{
    cam_thread::settings::StaticWidget,
    error::CatchAppResult,
    ui::state::{camera::UICamera, AppState},
};
use eframe::{
    egui::{Button, CentralPanel, Context, Direction, Frame, Layout, TopBottomPanel},
    emath::Align,
};
use gcam_lib::{error::AppResult, utils::geom::fit_size_into};

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
                            "⏸ Stop live view"
                        } else {
                            "▶ Start live view"
                        }),
                    )
                    .clicked()
                {
                    set_live_view = Some(!camera.live_view_enabled);
                }
            });

            ui.with_layout(Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                Frame::dark_canvas(ui.style()).show(ui, |ui| {
                    TopBottomPanel::bottom("camera_preview_config")
                        .show_inside(ui, |_| camera_focus_ui(camera));
                    if let Some(preview) = &state.last_preview_capture {
                        ui.image(preview, fit_size_into(preview.size_vec2(), ui.available_size()));
                    } else {
                        ui.label("No preview has been captured");
                    }
                });
            });
        })
    });

    if let Some(live_view_enabled) = set_live_view {
        let _ = state.set_live_view(live_view_enabled).catch(state);
    }
}

fn camera_focus_ui(camera: &mut UICamera) -> AppResult<()> {
    // TODO: Different camera models will have a different action for this
    if let Some(manualfocusdrive_setting) = camera
        .settings
        .as_ref()
        .and_then(|settings| settings.get_child("actions"))
        .and_then(|settings| settings.get_child("manualfocusdrive"))
    {
        if let StaticWidget::Range { .. } = &manualfocusdrive_setting.widget {
            // TODO
        } else {
            // TODO: Don't spam da logs
            log::error!(
                "Invalid widget type for manualfocusdrive: {:?}, expected Range widget",
                manualfocusdrive_setting.widget
            );
        }
    }

    Ok(())
}
