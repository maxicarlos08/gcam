use eframe::{epaint::Vec2, run_native, NativeOptions};
use gcam_lib::error::AppResult;

fn main() -> AppResult<()> {
    env_logger::init();

    let window_config =
        NativeOptions { min_window_size: Some(Vec2::new(600f32, 400f32)), ..Default::default() };

    run_native(
        "camera_gui",
        window_config,
        Box::new(move |_| Box::new(gcam::app::AppState::new().unwrap())),
    );

    Ok(())
}
