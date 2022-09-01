use crate::{
    messages::{self, CameraCommand, CameraCommandResponse, MessageFromThread, MessageToThread},
    settings::CameraSettings,
};
use gphoto2::{Camera, Context, Result};
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
