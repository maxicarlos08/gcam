use crate::cam_thread::messages::{MessageFromThread, MessageToThread, PreviewImage};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use gcam_lib::{error::AppResult, utils};
use gphoto2::{Camera, Context, Result};

pub struct CameraThreadState {
    pub context: Context,
    pub camera: Option<Camera>,
    pub capturing_live_view: bool,
}

pub(crate) fn camera_loop(
    message_receiver: Receiver<MessageToThread>,
    message_sender: Sender<MessageFromThread>,
) -> Result<()> {
    let mut camera_thread_state =
        CameraThreadState { context: Context::new()?, camera: None, capturing_live_view: false };

    loop {
        let action = if camera_thread_state.capturing_live_view {
            match message_receiver.try_recv() {
                Err(TryRecvError::Empty) => {
                    if let Err(err) = capture_preview(&mut camera_thread_state, &message_sender) {
                        message_sender.send(MessageFromThread::Error(err)).unwrap();
                    };

                    std::thread::sleep(std::time::Duration::from_millis(20));

                    continue;
                }
                Err(TryRecvError::Disconnected) => Err("Sender seems to be dead")?,
                Ok(action) => action,
            }
        } else {
            message_receiver.recv().map_err(|_| "Sender seems to be dead")?
        };

        match action {
            MessageToThread::SetLiveView(lv) => camera_thread_state.capturing_live_view = lv,
            MessageToThread::Break => break,
            MessageToThread::Closure(fun) => {
                message_sender
                    .send(match fun(&mut camera_thread_state) {
                        Ok(back_action) => MessageFromThread::Closure(back_action),
                        Err(err) => MessageFromThread::Error(err),
                    })
                    .unwrap();
            }
        }
    }

    Ok(())
}

fn capture_preview(
    state: &mut CameraThreadState,
    send: &Sender<MessageFromThread>,
) -> AppResult<()> {
    if let Some(camera) = &state.camera {
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
        state.capturing_live_view = false;
        send.send(MessageFromThread::Closure(Box::new(|state| {
            if let Some(camera) = state.camera.as_mut() {
                camera.live_view_enabled = false;
            }
        })))
        .unwrap();
    }

    Ok(())
}
