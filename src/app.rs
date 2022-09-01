use crate::{
    error::{ToUIError, UiError},
    settings::Settings,
    ui::{components, views},
};
use camera_controller::{
    messages::{
        CameraCommand, CameraCommandResponse, CameraInfo, MessageFromThread, MessageToThread,
    },
    settings::CameraSettings,
    CameraThread,
};
use eframe::{
    egui::Context,
    epaint::{TextureHandle, TextureId},
};
use gcam_lib::error::AppResult;
use std::{collections::HashMap, time::Duration};

const REPAINT_INTERVAL: Duration = Duration::from_millis(100);

pub type ModifiedSettingsMap = HashMap<u32, (u32, CameraSettings)>;

pub struct VisiblePanes {
    pub camera_info: bool,
    pub camera_settings: bool,
    pub camera_media: bool,
}

#[derive(Debug)]
pub struct UICamera {
    pub info: CameraInfo,
    pub settings: Option<CameraSettings>,
    pub settings_status_id: Option<u32>,
    pub modified_settings: ModifiedSettingsMap,
    pub live_view_enabled: bool,
}

#[derive(Default)]
pub struct Dialogs {
    pub camera_info_text: bool,
    pub settings: bool,
}

pub struct AppState {
    pub camera_list: Vec<(String, String)>,
    pub current_camera: Option<(String, String)>,
    pub camera: Option<UICamera>,
    pub panes: VisiblePanes,
    pub open_dialogs: Dialogs,
    pub settings: Settings,
    pub last_preview_capture: Option<TextureHandle>,
    errors: Vec<UiError>,
    camera_thread: CameraThread,
    first_load: bool,
}

impl Default for VisiblePanes {
    fn default() -> Self {
        Self {
            camera_info: true,
            camera_settings: true,
            camera_media: true,
        }
    }
}

impl UICamera {
    pub fn modify_setting(&mut self, section_id: u32, setting: CameraSettings) {
        self.modified_settings
            .insert(setting.id, (section_id, setting));
    }

    pub fn discard_settings(&mut self) {
        self.modified_settings.clear();
    }
}

impl AppState {
    pub fn new() -> AppResult<Self> {
        let camera_thread = CameraThread::start();

        let settings = if let Ok(Some(settings)) = Settings::get_user_settings() {
            settings
        } else {
            let settings = Settings::default();
            settings.save()?;
            settings
        };

        let _self = Self {
            first_load: true,
            camera_thread,
            current_camera: Default::default(),
            camera: Default::default(),
            camera_list: Default::default(),
            open_dialogs: Default::default(),
            panes: Default::default(),
            last_preview_capture: None,
            errors: vec![],
            settings,
        };

        _self.update_cameras()?;

        Ok(_self)
    }

    pub fn update_cameras(&self) -> AppResult<()> {
        self.camera_thread.send(MessageToThread::ListCameras)?;
        Ok(())
    }

    pub fn close_camera(&mut self) -> AppResult<()> {
        self.camera_thread.send(MessageToThread::CloseCamera)?;
        Ok(())
    }

    pub fn use_camera(&mut self, model: String, port: String) -> AppResult<()> {
        if self.camera.is_some() {
            self.close_camera()?;
        }

        self.camera = None;
        self.camera_thread
            .send(MessageToThread::UseCamera(model, port))?;
        self.reload_settings()?;
        Ok(())
    }

    pub fn reload_settings(&self) -> AppResult<()> {
        self.camera_thread
            .send(MessageToThread::CameraCommand(CameraCommand::GetConfig))?;
        Ok(())
    }

    pub fn apply_settings(&mut self) -> AppResult<()> {
        if let Some(UICamera {
            settings: Some(settings),
            modified_settings,
            ..
        }) = &mut self.camera
        {
            for (setting_id, (section_id, modified)) in modified_settings.drain() {
                self.camera_thread.send(MessageToThread::CameraCommand(
                    CameraCommand::SetConfig(modified.clone()),
                ))?;

                settings
                    .children
                    .get_mut(&section_id)
                    .ok_or("Section not found")?
                    .children
                    .insert(setting_id, modified);
            }
        }

        Ok(())
    }

    pub fn set_live_view(&mut self, live_view: bool) -> AppResult<()> {
        if let Some(UICamera {
            live_view_enabled, ..
        }) = &mut self.camera
        {
            *live_view_enabled = live_view;
            self.camera_thread
                .send(MessageToThread::CameraCommand(CameraCommand::SetLiveView(
                    live_view,
                )))?;
        }

        Ok(())
    }

    pub fn show_error(&mut self, error: UiError) {
        self.errors.push(error);
    }
}

impl AppState {
    fn process_events_from_camera_thread(&mut self, ctx: &Context) -> AppResult<()> {
        // Use the first available camera if this is the first run
        let mut use_first_camera = false;
        let mut errors: Option<Vec<UiError>> = None;

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
                        self.camera = Some(UICamera {
                            info: camera,
                            settings: None,
                            settings_status_id: None,
                            modified_settings: Default::default(),
                            live_view_enabled: false,
                        });
                    } else {
                        self.current_camera = None;
                        self.camera = None;
                    }
                }
                MessageFromThread::CameraCommandResponse(response) => match response {
                    CameraCommandResponse::CameraConfig(config) => {
                        if let Some(camera) = &mut self.camera {
                            camera.modified_settings.clear();

                            if let Some((id, _)) = config
                                .children
                                .iter()
                                .find(|(_, child)| child.name == "status")
                            {
                                camera.settings_status_id = Some(*id);
                            }

                            camera.settings = Some(config);
                        }
                    }
                    CameraCommandResponse::LiveView(capturing_previews) => {
                        if let Some(camera) = &mut self.camera {
                            camera.live_view_enabled = capturing_previews;
                        }
                    }
                },
                MessageFromThread::Error(err) => {
                    eprintln!("Error from camera thread: {:?}", err);
                    let ui_error = err.to_ui_error();
                    if let Some(errors) = &mut errors {
                        errors.push(ui_error)
                    } else {
                        errors = Some(vec![ui_error])
                    }
                }
                MessageFromThread::PreviewCapture(capture) => {
                    let texture_handle =
                        ctx.load_texture("preview_image", capture.0, Default::default());
                    self.last_preview_capture = Some(texture_handle);
                }
            }
        }

        if use_first_camera {
            self.use_camera(
                self.camera_list[0].0.to_owned(),
                self.camera_list[0].1.to_owned(),
            )?;
        }

        if let Some(errors) = errors {
            for error in errors {
                self.show_error(error);
            }
        }

        Ok(())
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        if let Err(err) = self.process_events_from_camera_thread(ctx) {
            self.errors.push(err.to_ui_error())
        }

        components::top_menu::show(ctx, frame, self);
        components::bottom_bar::show(ctx, frame);
        views::main::show(ctx, self);

        if self.open_dialogs.settings {
            views::settings::show(ctx, self);
        }

        {
            let mut pop_error = false;

            for error in &self.errors {
                if components::error::show(ctx, &error) {
                    pop_error = true;
                }
            }

            if pop_error {
                self.errors.pop();
            }
        }

        ctx.request_repaint_after(REPAINT_INTERVAL);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.camera_thread.stop().unwrap();
    }
}
