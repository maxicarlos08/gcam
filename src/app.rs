use crate::ui::{main, top_menu};
use camera_controller::{
    messages::{CameraCommandResponse, CameraInfo, MessageFromThread, MessageToThread},
    CameraThread,
};
use eframe::{egui::Context};
use std::{sync::mpsc::SendError, time::Duration, collections::VecDeque};

const REPAINT_INTERVAL: Duration = Duration::from_millis(100);

pub struct VisiblePanes {
    pub camera_info: bool,
}

#[derive(Debug)]
pub struct UICamera {
    pub info: CameraInfo,
}

#[derive(Default)]
pub struct Dialogs {
    pub camera_info_text: bool,
}

pub struct AppState {
    pub camera_list: Vec<(String, String)>,
    pub current_camera: Option<(String, String)>,
    pub camera: Option<UICamera>,
    pub panes: VisiblePanes,
    pub open_dialogs: Dialogs,
    errors: VecDeque<gphoto2::Error>,
    camera_thread: CameraThread,
    first_load: bool,
}

impl Default for VisiblePanes {
    fn default() -> Self {
        Self {
            camera_info: true,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        
        let camera_thread = CameraThread::start();
        
        let _self = Self {
            first_load: true,
            camera_thread,
            current_camera: Default::default(),
            camera: Default::default(),
            camera_list: Default::default(),
            open_dialogs: Default::default(),
            panes: Default::default(),
        errors: Default::default(),
    };
    
    _self.update_cameras();
    
    _self
    
}
}

impl AppState {
    pub fn update_cameras(&self) -> Result<(), SendError<MessageToThread>> {
        self.camera_thread.send(MessageToThread::ListCameras)
    }

    pub fn close_camera(&mut self) -> Result<(), SendError<MessageToThread>> {
        self.camera_thread.send(MessageToThread::CloseCamera)
    }

    pub fn use_camera(
        &mut self,
        model: String,
        port: String,
    ) -> Result<(), SendError<MessageToThread>> {
        if self.camera.is_some() {
            self.close_camera();
        }

        self.camera = None;
        self.camera_thread
            .send(MessageToThread::UseCamera(model, port))
    }

    fn process_events_from_camera_thread(&mut self) {
        // Use the first available camera if this is the first run
        let mut use_first_camera = false;

        for event in self.camera_thread.receiver().try_iter() {
            match event {
                MessageFromThread::CameraList(list) => {
                    self.camera_list = list;
                    if self.first_load && !self.camera_list.is_empty() {
                        use_first_camera = true;
                        self.first_load = false;
                    }
                }
                MessageFromThread::CameraOpen(camera) => {
                    if let Some(camera) = camera {
                        self.current_camera =
                            Some((camera.model.to_owned(), camera.port.to_owned()));
                        self.camera = Some(UICamera { info: camera });
                    } else {
                        self.current_camera = None;
                        self.camera = None;
                    }
                }
                MessageFromThread::CameraCommandResponse(response) => match response {
                    CameraCommandResponse::CameraSetting(_, _) => todo!(),
                },
                MessageFromThread::Error(err) => println!("Error from camera thread: {}", err),
                MessageFromThread::PreviewCapture(_) => todo!(),
            }
        }

        if use_first_camera {
            self.use_camera(
                self.camera_list[0].0.to_owned(),
                self.camera_list[0].1.to_owned(),
            );
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        self.process_events_from_camera_thread();

        top_menu::show(ctx, frame, self);
        main::show(ctx, self);

        ctx.request_repaint_after(REPAINT_INTERVAL);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.camera_thread.stop().unwrap();
    }
}
