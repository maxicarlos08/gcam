use crate::{
    app::{AppState, ModifiedSettingsMap},
    error::CatchAppResult,
    settings::Settings,
};
use camera_controller::settings::CameraSettings;
use eframe::{
    egui::{
        ComboBox, Context, DragValue, Frame, Layout, ScrollArea, SidePanel, Slider, TopBottomPanel,
        Ui,
    },
    emath::Align,
    epaint::Color32,
};
use gphoto2::widget::{WidgetType, WidgetValue};

pub fn show(ctx: &Context, state: &mut AppState) {
    let mut changed_setting = None;
    let mut apply_settings = false;
    let mut discard_settings = false;

    if let Some(camera) = &state.camera {
        SidePanel::right("camera_settings_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Camera settings");

                if ui
                    .with_layout(Layout::right_to_left(Align::Center), |ui| ui.button("↻"))
                    .inner
                    .clicked()
                {
                    state.reload_settings().unwrap();
                }
            });
            ui.separator();

            let mut set_setting = |setting: CameraSettings, section_id: u32| {
                changed_setting = Some((setting, section_id))
            };

            if let Some(camera_settings) = &camera.settings {
                display_settings_root(
                    ui,
                    &state.settings,
                    camera_settings,
                    &camera.modified_settings,
                    &mut set_setting,
                    &mut apply_settings,
                    &mut discard_settings,
                );
            } else {
                ui.centered_and_justified(|ui| ui.label("Loading settings..."));
            }
        });
    }

    if apply_settings {
        let _ = state.apply_settings().catch(state);
    } else if let Some(camera) = &mut state.camera {
        if let Some((setting, section_id)) = changed_setting {
            camera.modify_setting(section_id, setting);
        } else if discard_settings {
            camera.discard_settings();
        }
    }
}

#[inline]
fn display_settings_root<S: FnMut(CameraSettings, u32)>(
    ui: &mut Ui,
    app_settings: &Settings,
    camera_settings: &CameraSettings,
    modified_settings: &ModifiedSettingsMap,
    set_setting: &mut S,
    apply_settings: &mut bool,
    discard_settings: &mut bool,
) {
    if camera_settings.is_root() {
        TopBottomPanel::bottom("camera_settings_apply").show_inside(ui, |ui| {
            ui.add_enabled_ui(!modified_settings.is_empty(), |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.button("Apply").clicked() {
                        *apply_settings = true;
                    }
                    if ui.button("Cancel").clicked() {
                        *discard_settings = true;
                    }
                })
            })
        });

        ScrollArea::vertical().show(ui, |ui| {
            for section in camera_settings.children.values() {
                if app_settings.dev_settings.exclude_settings.contains(&section.name) {
                    continue;
                }

                let section_name = section.label.as_ref().unwrap_or(&section.name);

                ui.collapsing(section_name, |ui| {
                    display_settings_section(
                        ui,
                        app_settings,
                        section,
                        modified_settings,
                        set_setting,
                    )
                });
            }
        });
    }
}

#[inline]
fn display_settings_section<S: FnMut(CameraSettings, u32)>(
    ui: &mut Ui,
    app_settings: &Settings,
    section: &CameraSettings,
    modified_settings: &ModifiedSettingsMap,
    set_setting: &mut S,
) {
    if section.is_section() {
        for setting in section.children.values() {
            if app_settings.dev_settings.exclude_settings.contains(&setting.name) {
                continue;
            }

            let mut changed = false;
            let (mut setting, modified) = {
                match modified_settings.get(&setting.id) {
                    Some((_, setting)) => (setting.to_owned(), true),
                    None => (setting.to_owned(), false),
                }
            };

            ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                display_setting(ui, &mut setting, &mut changed, modified);
            });

            if changed {
                set_setting(setting, section.id);
            }
        }
    }
}

#[inline]
fn display_setting(ui: &mut Ui, setting: &mut CameraSettings, changed: &mut bool, modified: bool) {
    let mut group = Frame::group(ui.style());
    if modified {
        group.stroke.color = Color32::LIGHT_GRAY;
    }

    group.show(ui, |ui| {
        let setting_label = setting.label.as_ref().unwrap_or(&setting.name);
        ui.label(setting_label);

        if let (Some(value), widget_type) = (&mut setting.value, &setting.widget_type) {
            ui.separator();

            ui.add_enabled_ui(!setting.readonly, |ui| match value {
                WidgetValue::Text(text) => {
                    if ui.text_edit_singleline(text).changed() {
                        *changed = true;
                    }
                }
                WidgetValue::Menu(selected) => {
                    if let WidgetType::Menu { choices, .. } = widget_type {
                        ComboBox::from_id_source(setting_label).selected_text(&*selected).show_ui(
                            ui,
                            |ui| {
                                for choice in choices {
                                    if ui
                                        .selectable_value(selected, choice.to_owned(), choice)
                                        .changed()
                                    {
                                        *changed = true;
                                    }
                                }
                            },
                        );
                    } else {
                        ui.label(format!("Invalid widget value: {:?}", value));
                    }
                }
                WidgetValue::Toggle(toggled) => {
                    // TODO: I bet this can be done better
                    ComboBox::from_id_source(setting_label)
                        .selected_text(match toggled {
                            Some(true) => "Yes",
                            Some(false) => "No",
                            None => "Unknown",
                        })
                        .show_ui(ui, |ui| {
                            for option in [true, false] {
                                if ui
                                    .selectable_value(
                                        toggled,
                                        Some(option),
                                        if option { "Yes" } else { "No" },
                                    )
                                    .changed()
                                {
                                    *changed = true
                                }
                            }
                        });
                }
                WidgetValue::Range(current) => {
                    if let WidgetType::Range { min, max, increment } = widget_type {
                        if ui
                            .add(Slider::new(current, *min..=*max).step_by(*increment as f64))
                            .changed()
                        {
                            *changed = true;
                        }
                    } else {
                        ui.label(format!("Invalid widget value: {:?}", value));
                    }
                }
                WidgetValue::Date(seconds) => {
                    if ui.add(DragValue::new(seconds)).changed() {
                        *changed = true;
                    }
                }
            });
        }
    });
}
