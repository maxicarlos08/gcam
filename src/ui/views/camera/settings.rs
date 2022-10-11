use crate::{
  cam_thread::settings::{display_radio_choice, CameraSettings, RadioChoice, StaticWidget},
  error::CatchAppResult,
  settings::Settings,
  ui::{
    state::{camera::ModifiedSettingsMap, AppState},
    widgets,
  },
};
use eframe::{
  egui::{
    ComboBox, Context, DragValue, Frame, Layout, ScrollArea, SidePanel, Slider, TopBottomPanel, Ui,
  },
  emath::Align,
  epaint::Color32,
};

pub fn show(ctx: &Context, state: &mut AppState) {
  let mut changed_setting = None;
  let mut apply_settings = false;
  let mut discard_settings = false;

  if let Some(camera) = &state.camera {
    SidePanel::right("camera_settings_panel").show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.heading("Camera settings");

        if ui.with_layout(Layout::right_to_left(Align::Center), |ui| ui.button("↻")).inner.clicked()
        {
          state.reload_settings().unwrap();
        }
      });
      ui.separator();

      let mut set_setting =
        |setting: CameraSettings, section_id: i32| changed_setting = Some((setting, section_id));

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
fn display_settings_root(
  ui: &mut Ui,
  app_settings: &Settings,
  camera_settings: &CameraSettings,
  modified_settings: &ModifiedSettingsMap,
  set_setting: &mut impl FnMut(CameraSettings, i32),
  apply_settings: &mut bool,
  discard_settings: &mut bool,
) {
  if let StaticWidget::Group { children, .. } = &camera_settings.widget {
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
      for section in children.values() {
        if app_settings.dev_settings.exclude_settings.contains(&section.name) {
          continue;
        }

        ui.collapsing(&section.name, |ui| {
          display_settings_section(ui, app_settings, section, modified_settings, set_setting)
        });
      }
    });
  }
}

#[inline]
fn display_settings_section(
  ui: &mut Ui,
  app_settings: &Settings,
  section: &CameraSettings,
  modified_settings: &ModifiedSettingsMap,
  set_setting: &mut impl FnMut(CameraSettings, i32),
) {
  if let StaticWidget::Group { children, .. } = &section.widget {
    for setting in children.values() {
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
    ui.label(&setting.label);

    ui.separator();

    ui.add_enabled_ui(!setting.readonly, |ui| match &mut setting.widget {
      StaticWidget::Text(text) => {
        *changed = ui.text_edit_singleline(text).changed();
      }
      StaticWidget::Radio { choices, choice } => {
        ComboBox::from_id_source(&setting.label)
          .selected_text(display_radio_choice(choices, choice))
          .show_ui(ui, |ui| {
            let mut selected_choice_i = 0;

            for (choice_i, choice_text) in choices.iter().enumerate() {
              // TODO: To not use to_owned, maybe use choice a index here?
              if ui.selectable_value(&mut selected_choice_i, choice_i, choice_text).changed() {
                *changed = true;
              }
            }

            if *changed {
              *choice = RadioChoice::Indexed(selected_choice_i);
            }
          });
      }
      StaticWidget::Toggle { undefined, value } => {
        *changed = widgets::toggle::toggle_widget(undefined, value, ui).changed();
      }
      StaticWidget::Range { range, ref mut value, step } => {
        *changed = ui.add(Slider::new(value, range.to_owned()).step_by(*step as f64)).changed();
      }
      StaticWidget::Date { timestamp } => *changed = ui.add(DragValue::new(timestamp)).changed(),
      StaticWidget::Button => {
        ui.label("TODO: button"); // TODO
      }
      StaticWidget::Group { .. } => {
        unreachable!("Only a window widget should have group widget children")
      }
    });
  });
}
