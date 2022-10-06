pub mod camera;
pub mod dialogs;

use self::{camera::UICamera, dialogs::Dialogs};
use crate::{
    cam_thread::{
        messages::{FromCameraThreadClosure, MessageFromThread, MessageToThread},
        settings::StaticWidget,
        CameraThread,
    },
    camera::info::CameraInfo,
    error::{ToUIError, UiError},
    settings::Settings,
};
use eframe::egui::Context;
use epaint::TextureHandle;
use gcam_lib::error::AppResult;
use gphoto2::{
    list::CameraDescriptor,
    widget::{
        ButtonWidget, DateWidget, RadioWidget, RangeWidget, TextWidget, ToggleWidget, Widget,
    },
};

pub struct VisiblePanes {
    pub camera_info: bool,
    pub camera_settings: bool,
    pub camera_media: bool,
}

pub struct AppState {
    pub camera_list: Vec<CameraDescriptor>,
    pub current_camera: Option<CameraDescriptor>,
    pub camera: Option<UICamera>,
    pub panes: VisiblePanes,
    pub open_dialogs: Dialogs,
    pub settings: Settings,
    pub last_preview_capture: Option<TextureHandle>,
    pub errors: Vec<UiError>,
    camera_thread: CameraThread,
    first_load: bool,
}

impl Default for VisiblePanes {
    fn default() -> Self {
        Self { camera_info: true, camera_settings: true, camera_media: true }
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
        self.camera_thread.send_fn(Box::new(|cam_state| {
            let cameras: Vec<_> = cam_state.context.list_cameras()?.collect();

            Ok(Box::new(move |state| {
                if state.first_load && !cameras.is_empty() {
                    state.use_camera(cameras[0].clone()).unwrap();
                }
                state.camera_list = cameras;
            }))
        }))
    }

    pub fn close_camera(&mut self) -> AppResult<()> {
        self.camera_thread.send_fn(Box::new(|cam_state| {
            drop(cam_state.camera.take()); // Goodbye camera

            Ok(Box::new(|state| {
                state.camera = None;
                state.current_camera = None;
            }))
        }))
    }

    pub fn use_camera(&mut self, descriptor: CameraDescriptor) -> AppResult<()> {
        if self.camera.is_some() {
            self.close_camera()?;
        }

        self.camera = None;
        self.camera_thread.send_fn(Box::new(move |cam_state| {
            let new_camera = cam_state.context.get_camera(&descriptor)?;
            let info = CameraInfo {
                abilities: new_camera.abilities(),
                about: new_camera.about().ok(),
                manual: new_camera.manual().ok(),
                model: descriptor.model.clone(),
                port: descriptor.port.clone(),
                storages: new_camera.storages()?,
                summary: new_camera.summary().ok(),
            };

            cam_state.camera = Some(new_camera);

            Ok(Box::new(move |state| {
                state.current_camera = Some(descriptor.clone());
                state.camera = Some(UICamera {
                    info,
                    settings: None,
                    block_config: false,
                    modified_settings: Default::default(),
                    live_view_enabled: false,
                });
            }))
        }))?;
        self.reload_settings()?;
        Ok(())
    }

    pub fn reload_settings(&self) -> AppResult<()> {
        self.camera_thread.send_fn(Box::new(|cam_state| {
            if let Some(camera) = &cam_state.camera {
                let config = Widget::Group(camera.config()?).try_into()?;

                Ok(Box::new(move |state| {
                    if let Some(camera) = &mut state.camera {
                        camera.settings = Some(config);
                    }
                }))
            } else {
                Err("There as no camera to reload configuration")?
            }
        }))
    }

    pub fn apply_settings(&mut self) -> AppResult<()> {
        if let Some(UICamera { modified_settings, block_config, .. }) = &mut self.camera {
            let modified_settings = modified_settings.drain().collect::<Vec<_>>();
            *block_config = true;
            self.camera_thread.send_fn(Box::new(move |cam_state| {
                if let Some(camera) = &cam_state.camera {
                    for (_, (_, modified)) in modified_settings {
                        let cam_widget: Widget = camera.config_key(&modified.name)?;

                        // TODO: Clean this code up using macros maybe?
                        let widget = match modified.widget {
                            StaticWidget::Group { .. } => unreachable!(),
                            StaticWidget::Text(text) => {
                                let text_widget = cam_widget.try_into::<TextWidget>()?;
                                text_widget.set_value(&text)?;
                                Widget::Text(text_widget)
                            }
                            StaticWidget::Range { value, .. } => {
                                let range_widget = cam_widget.try_into::<RangeWidget>()?;
                                range_widget.set_value(value);
                                Widget::Range(range_widget)
                            }
                            StaticWidget::Toggle { value, .. } => {
                                let toggle_widget = cam_widget.try_into::<ToggleWidget>()?;
                                toggle_widget.set_toggled(value);
                                Widget::Toggle(toggle_widget)
                            }
                            StaticWidget::Radio { choice, .. } => {
                                let radio_widget = cam_widget.try_into::<RadioWidget>()?;
                                radio_widget.set_choice(&choice)?;
                                Widget::Radio(radio_widget)
                            }
                            StaticWidget::Button => {
                                let button = cam_widget.try_into::<ButtonWidget>()?;
                                button.press(camera)?;
                                Widget::Button(button)
                            }
                            StaticWidget::Date { timestamp } => {
                                let time_widget = cam_widget.try_into::<DateWidget>()?;
                                time_widget.set_timestamp(timestamp);
                                Widget::Date(time_widget)
                            }
                        };

                        camera.set_config(&widget)?;
                    }
                }

                Ok(Box::new(|state| {
                    if let Some(camera) = &mut state.camera {
                        camera.block_config = false
                    }
                }))
            }))
        } else {
            Ok(())
        }
    }

    pub fn set_live_view(&mut self, live_view: bool) -> AppResult<()> {
        if let Some(UICamera { live_view_enabled, .. }) = &mut self.camera {
            *live_view_enabled = live_view;
            self.camera_thread.send(MessageToThread::SetLiveView(live_view))?;
        }

        Ok(())
    }

    pub fn show_error(&mut self, error: UiError) {
        self.errors.push(error);
    }
}

impl AppState {
    pub(crate) fn process_events_from_camera_thread(&mut self, ctx: &Context) -> AppResult<()> {
        let mut errors: Option<Vec<UiError>> = None;

        let mut closures: Option<Vec<FromCameraThreadClosure>> = None;
        for event in self.camera_thread.receiver().try_iter() {
            match event {
                MessageFromThread::Error(err) => {
                    log::error!("Got error from camera thread: {:?}", err);

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
                MessageFromThread::Closure(cls) => {
                    closures.get_or_insert(vec![]).push(cls);
                }
            }
        }

        if let Some(closures) = closures {
            for closure in closures {
                closure(self);
            }
        }

        if let Some(errors) = errors {
            for error in errors {
                self.show_error(error);
            }
        }

        Ok(())
    }

    pub(crate) fn stop_camera_thread(&mut self) {
        self.camera_thread.stop().unwrap()
    }
}
