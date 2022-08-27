use crate::app::AppState;
use eframe::{
    egui::{self, global_dark_light_mode_switch, menu::bar, Context, RichText, TopBottomPanel},
    epaint::Color32,
};

pub fn show(ctx: &Context, frame: &mut eframe::Frame, state: &mut AppState) {
    TopBottomPanel::top("app_menu").show(ctx, |ui| {
        bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                let quit_button = ui.button("Quit");

                if quit_button.clicked() {
                    frame.close()
                }
            });

            ui.menu_button("Camera", |ui| {
                let refresh_camera = ui.button("Refresh cameras");

                ui.separator();

                if state.camera_list.is_empty() {
                    ui.label(RichText::new("No cameras detected").color(Color32::DARK_GRAY));
                } else {
                    ui.vertical(|ui| {
                        let mut selected_camera = None;
                        for camera in &state.camera_list {
                            if ui
                                .radio_value(
                                    &mut state.current_camera,
                                    Some(camera.clone()), // FIXME: Get rid of this clone
                                    &camera.0,
                                )
                                .changed()
                                && state.current_camera != selected_camera
                            {
                                selected_camera = Some(camera.clone());
                            }
                        }

                        if let Some(camera) = selected_camera {
                            state.use_camera(camera.0, camera.1);
                        }
                    });
                }

                if state.current_camera.is_some() {
                    ui.separator();

                    let close_camera = ui.add_enabled(
                        state.camera.is_some(),
                        egui::widgets::Button::new("Close camera"),
                    );

                    if close_camera.clicked() {
                        state.close_camera();
                    }
                }

                if refresh_camera.clicked() {
                    state.update_cameras();
                }
            });

            ui.menu_button("View", |ui| {
                ui.add_enabled_ui(state.camera.is_some(), |ui| {
                    ui.menu_button("Panels", |ui| {
                        ui.toggle_value(&mut state.panes.camera_info, "Camera info");
                    })
                })
            });

            ui.with_layout(
                egui::Layout::right_to_left(eframe::emath::Align::Center),
                global_dark_light_mode_switch,
            );
        })
    });
}
