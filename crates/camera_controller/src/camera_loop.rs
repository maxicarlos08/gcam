use std::time::Instant;

use crate::{
    messages::{
        self, CameraCommand, CameraCommandResponse, MessageFromThread, MessageToThread,
        PreviewImage,
    },
    settings::CameraSettings,
};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use gcam_lib::{error::AppResult, utils};
use gphoto2::{Camera, Context, Result};

pub(crate) fn camera_loop(
    recv_message: Receiver<MessageToThread>,
    send_message: Sender<MessageFromThread>,
) -> Result<()> {
    let context = Context::new()?;
    let mut camera: Option<Camera> = None;
    let mut capturing_previews = false;

    loop {
        let action = if capturing_previews {
            match recv_message.try_recv() {
                Err(TryRecvError::Empty) => {
                    if let Err(err) =
                        capture_preview(&camera, &mut capturing_previews, &send_message)
                    {
                        send_message.send(MessageFromThread::Error(err)).unwrap();
                    };

                    std::thread::sleep(std::time::Duration::from_millis(20));

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
) -> AppResult<bool> {
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
            let new_camera = context.get_camera(&model, &port).map(|err| {
                send.send(MessageFromThread::CameraOpen(None)).unwrap();
                err
            })?;
            let camera_info = messages::CameraInfo {
                model,
                port,
                about: new_camera.about().map(Into::into).ok(),
                manual: new_camera.manual().map(Into::into).ok(),
                summary: new_camera.summary().map(Into::into).ok(),
                abilities: new_camera.abilities()?,
                storages: new_camera.storages()?,
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
                    CameraCommand::SetLiveView(lv) => {
                        if camera.abilities()?.camera_operations().capture_preview() {
                            *capturing_previews = lv
                        }
                    }
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

fn capture_preview(
    camera: &Option<Camera>,
    capturing_previews: &mut bool,
    send: &Sender<MessageFromThread>,
) -> AppResult<()> {
    if let Some(camera) = &camera {
        match camera.capture_preview() {
            Ok(preview) => {
                let data = preview.get_data()?;
                send.send(MessageFromThread::PreviewCapture(PreviewImage(
                    utils::image::decode_image(&data)?,
                )))
                .unwrap();
            }
            Err(error) => {
                send.send(MessageFromThread::Error(error.into())).unwrap();
            }
        }
    } else {
        *capturing_previews = false;
        send.send(MessageFromThread::CameraCommandResponse(
            CameraCommandResponse::LiveView(false),
        ))
        .unwrap();
    }

    Ok(())
}
