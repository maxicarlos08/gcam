use eframe::{run_native, NativeOptions};

fn main() {
    let window_config = NativeOptions::default();
    run_native(
        "camera_gui",
        window_config,
        Box::new(|_| Box::new(gcam::app::AppState::default())),
    );
}
