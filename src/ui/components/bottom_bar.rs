use eframe::{
    egui::{Context, TopBottomPanel},
    Frame,
};

pub fn show(ctx: &Context, frame: &Frame) {
    TopBottomPanel::bottom("bottom_panle").show(ctx, |ui| {
        if let Some(perf) = frame.info().cpu_usage {
            ui.label(format!("Perf: {:.2}ms", perf * 1000f32));
        }
    });
}
