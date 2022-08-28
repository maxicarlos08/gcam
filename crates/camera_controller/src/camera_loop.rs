use crate::{
    messages::{
        self, CameraCommand, CameraCommandResponse, CameraStatus, MessageFromThread,
        MessageToThread,
    },
    settings::CameraSettings,
};
use gphoto2::{widget::WidgetValue, Camera, Context, Result};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

pub(crate) fn camera_loop(
    recv_message: Receiver<MessageToThread>,
    send_message: Sender<MessageFromThread>,
) -> Result<()> {
    let context = Context::new()?;
    let mut camera = None;
    let mut capturing_previews = false;

    loop {
        let action = if capturing_previews {
            match recv_message.try_recv() {
                Err(TryRecvError::Empty) => {
                    // TODO: Capture preview and send to parent
                    continue;
                }
                Err(TryRecvError::Disconnected) => Err("Sender seems to be dead")?,
                Ok(action) => action,
            }
        } else {
            recv_message.recv().map_err(|_| "Sender seems to be dead")?
        };

        match exec_action(
            action,
            &send_message,
            &context,
            &mut camera,
            &mut capturing_previews,
        ) {
            Ok(true) => break,
            Ok(_) => {}
            Err(error) => {
                send_message.send(MessageFromThread::Error(error)).unwrap();
            }
        }
    }

    Ok(())
}

fn exec_action<'a, 'b: 'a>(
    action: MessageToThread,
    send: &Sender<MessageFromThread>,
    context: &'b Context<'b>,
    camera: &'a mut Option<Camera<'b>>,
    capturing_previews: &mut bool,
) -> Result<bool> {
    match action {
        MessageToThread::Break => return Ok(true),
        MessageToThread::ListCameras => {
            let cameras = context
                .list_cameras()?
                .to_vec()?
                .iter()
                .map(|(name, port)| (name.to_string(), port.to_string()))
                .collect();

            send.send(MessageFromThread::CameraList(cameras)).unwrap();
        }
        MessageToThread::UseCamera(model, port) => {
            macro_rules! get_status_config {
                ($name:expr, $root:expr) => {
                    $root
                        .get_child_by_name($name)
                        .ok()
                        .map(|config| config.value().ok())
                        .flatten()
                        .map(|value| value.0)
                        .flatten()
                };
            }

            let new_camera = context.get_camera(&model, &port)?;
            let camera_info = messages::CameraInfo {
                model,
                port,
                about: new_camera.about().map(Into::into).ok(),
                manual: new_camera.manual().map(Into::into).ok(),
                summary: new_camera.summary().map(Into::into).ok(),
                abilities: new_camera.abilities()?,
                storages: new_camera.storages()?,
                status: {
                    // TODO: Move this code somewhere else and make it compatible for more cameras
                    let status_config = new_camera.config()?.get_child_by_name("status")?;

                    CameraStatus {
                        serial_number: if let Some(WidgetValue::Text(serial)) =
                            get_status_config!("serialnumber", status_config)
                        {
                            Some(serial)
                        } else {
                            None
                        },
                        manufacturer: if let Some(WidgetValue::Text(manufacturer)) =
                            get_status_config!("manufacturer", status_config)
                        {
                            Some(manufacturer)
                        } else {
                            None
                        },
                        model: if let Some(WidgetValue::Text(model)) =
                            get_status_config!("cameramodel", status_config)
                        {
                            Some(model)
                        } else {
                            None
                        },
                        ac_power: if let Some(WidgetValue::Menu(ac_power)) =
                            get_status_config!("acpower", status_config)
                        {
                            Some(ac_power == "On")
                        } else {
                            None
                        },
                        battery_level: if let Some(WidgetValue::Text(battery_level)) =
                            get_status_config!("batterylevel", status_config)
                        {
                            let battery_value: f32 = battery_level
                                .split("%")
                                .nth(0)
                                .ok_or("Invalid formatted battery level from camera")?
                                .parse()
                                .map_err(|_| "Battery level is not a number")?;

                            Some(battery_value / 100f32)
                        } else {
                            None
                        },
                    }
                },
            };

            *camera = Some(new_camera);

            send.send(MessageFromThread::CameraOpen(Some(camera_info)))
                .unwrap();
        }
        MessageToThread::CloseCamera => {
            *camera = None;

            send.send(MessageFromThread::CameraOpen(None)).unwrap();
        }
        MessageToThread::CameraCommand(command) => {
            if let Some(camera) = camera {
                match command {
                    CameraCommand::SetLiveView(lv) => *capturing_previews = lv,
                    CameraCommand::GetConfig => {
                        let config: CameraSettings = camera.config()?.try_into()?;

                        send.send(MessageFromThread::CameraCommandResponse(
                            CameraCommandResponse::CameraConfig(config),
                        ))
                        .unwrap();
                    }
                    CameraCommand::SetConfig(config) => {
                        if let Some(value) = config.value {
                            let mut camera_config = camera.config_key(&config.name)?;
                            camera_config.set_value(value)?;
                            camera.set_config(&camera_config)?;
                        } else {
                            // TODO: Set configuration for Window or section
                            Err("Configuration has not value")?;
                        }
                    }
                }
            } else {
                Err("Cannot use camera command when there is no camera")?
            }
        }
    }

    Ok(false)
}
