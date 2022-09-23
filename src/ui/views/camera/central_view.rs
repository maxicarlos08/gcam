use crate::{
    app::{AppState, UICamera},
    error::CatchAppResult,
};
use eframe::{
    egui::{Button, CentralPanel, Context, Direction, Frame, Layout, TopBottomPanel},
    emath::Align,
};
use gcam_lib::{error::AppResult, utils::geom::fit_size_into};
use gphoto2::widget::WidgetType;

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
                        .show_inside(ui, |ui| camera_focus_ui(camera));
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
    if let Some(manualfocusdrive_setting) = camera
        .settings
        .as_ref()
        .map(|settings| {
            settings
                .children_id_names
                .get("actions")
                .map(|actions_id| settings.children.get(actions_id))
                .flatten()
        })
        .flatten()
        .map(|actions| {
            actions
                .children_id_names
                .get("manualfocusdrive")
                .map(|manual_focus_id| actions.children.get(manual_focus_id))
                .flatten()
        })
        .flatten()
    {
        if let WidgetType::Range { min, max, .. } = manualfocusdrive_setting.widget_type {
            // TODO
        } else {
            log::error!(
                "Invalid widget type for manualfocusdrive: {:?}, expected Range widget",
                manualfocusdrive_setting.widget_type
            );
        }
    }

    Ok(())
}
