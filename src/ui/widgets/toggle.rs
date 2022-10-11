use eframe::{egui::Sense, emath::lerp};
use epaint::{pos2, vec2};

fn toggle_widget_visual_value(undefined: bool, value: bool) -> f32 {
  match (undefined, value) {
    (true, _) => 0.5,
    (_, true) => 1.0,
    (_, false) => 0.0,
  }
}

/// Draws a toggle widget, the input is expected to be a [`StaticWidget::Toggle`]
///
/// Mostly copy-pasted from here: https://github.com/emilk/egui/blob/7803285221aa65cfdbcb2dc3d5efd2e318fa8f9b/crates/egui_demo_lib/src/demo/toggle_switch.rs#L17
pub fn toggle_widget(
  undefined: &mut bool,
  value: &mut bool,
  ui: &mut eframe::egui::Ui,
) -> eframe::egui::Response {
  // Decide the size of the widget
  let toggler_size = ui.spacing().interact_size.y * vec2(2., 1.);

  // Allocate the space for the widget and sense for click events
  let (rect, mut response) = ui.allocate_exact_size(toggler_size, Sense::click());

  // Check if the area has been clicked
  if response.clicked() {
    if *undefined {
      *value = true;
      *undefined = false;
    } else {
      *value = !*value;
    }

    response.mark_changed();
  }

  if ui.is_rect_visible(rect) {
    // How much "on" the toggle is visually
    let on_ness = ui.ctx().animate_value_with_time(
      response.id,
      toggle_widget_visual_value(*undefined, *value),
      ui.style().animation_time,
    );

    let visuals = ui.style().interact_selectable(&response, !*undefined && *value);
    let rect = rect.expand(visuals.expansion);
    let border_radius = 0.5 * rect.height();
    ui.painter().rect(rect, border_radius, visuals.bg_fill, visuals.bg_stroke);
    let circle_x = lerp((rect.left() + border_radius)..=(rect.right() - border_radius), on_ness);
    let circle_center = pos2(circle_x, rect.center().y);
    ui.painter().circle(circle_center, 0.75 * border_radius, visuals.bg_fill, visuals.fg_stroke);
  }

  response
}
