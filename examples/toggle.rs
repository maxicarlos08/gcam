//! This example is only used to debug the Toggle widget renderer

use eframe::{egui::CentralPanel, run_native};
use gcam::{cam_thread::settings::StaticWidget, ui::widgets::toggle};

struct ToggleExample {
  toggle: StaticWidget,
}

impl eframe::App for ToggleExample {
  fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
    CentralPanel::default().show(ctx, |ui| {
      if let StaticWidget::Toggle { undefined, value } = &mut self.toggle {
        if toggle::toggle_widget(undefined, value, ui).changed() {
          println!("The widget has changed: {:?}", self.toggle);
        }
      }
    });
  }
}

fn main() {
  run_native(
    "gcam_toggle_test",
    Default::default(),
    Box::new(|_cc| {
      Box::new(ToggleExample { toggle: StaticWidget::Toggle { undefined: true, value: false } })
    }),
  )
}
