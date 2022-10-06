mod camera_loop;
pub mod messages;
pub mod settings;

use crossbeam_channel::{unbounded, Receiver, SendError, Sender};
use gcam_lib::error::AppResult;
use std::thread::JoinHandle;

use self::messages::ToCameraThreadClosure;

pub struct CameraThread {
    handle: Option<JoinHandle<()>>,
    receiver: Receiver<messages::MessageFromThread>,
    sender: Sender<messages::MessageToThread>,
}

impl CameraThread {
    pub fn start() -> Self {
        let (to_thread_send, to_thread_recv) = unbounded();
        let (from_thread_send, from_thread_recv) = unbounded();

        let handle = std::thread::spawn(move || {
            camera_loop::camera_loop(to_thread_recv, from_thread_send)
                .expect("Failed to start camera loop"); // TODO handle this error
        });

        Self { handle: Some(handle), receiver: from_thread_recv, sender: to_thread_send }
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.sender.send(messages::MessageToThread::Break)?;

        if let Some(join_handle) = self.handle.take() {
            join_handle.join().unwrap();
        }

        Ok(())
    }

    pub fn send(
        &self,
        message: messages::MessageToThread,
    ) -> std::result::Result<(), SendError<messages::MessageToThread>> {
        self.sender.send(message)
    }

    pub fn receiver(&mut self) -> &mut Receiver<messages::MessageFromThread> {
        &mut self.receiver
    }

    pub fn send_fn(&self, _fn: ToCameraThreadClosure) -> AppResult<()> {
        self.send(messages::MessageToThread::Closure(_fn))?;
        Ok(())
    }
}
