mod camera_loop;
pub mod settings;

use std::{
    sync::mpsc::{Receiver, SendError, Sender},
    thread::JoinHandle,
};

pub mod messages {
    use crate::settings::CameraSettings;
    use gphoto2::filesys::StorageInfo;

    #[derive(Debug, PartialEq)]
    pub enum CameraCommandResponse {
        CameraConfig(CameraSettings),
    }

    #[derive(Debug, PartialEq)]
    pub enum MessageFromThread {
        PreviewCapture(Box<[u8]>),
        CameraCommandResponse(CameraCommandResponse),
        CameraList(Vec<(String, String)>),
        CameraOpen(Option<CameraInfo>),
        Error(gphoto2::Error),
    }

    #[derive(Debug)]
    pub struct CameraStatus {
        pub serial_number: Option<String>,
        pub manufacturer: Option<String>,
        pub model: Option<String>,
        pub ac_power: Option<bool>,
        pub battery_level: Option<f32>,
    }

    #[derive(Debug)]
    pub struct CameraInfo {
        pub status: CameraStatus,
        pub model: String,
        pub port: String,
        pub manual: Option<String>,
        pub summary: Option<String>,
        pub about: Option<String>,
        pub abilities: gphoto2::abilities::Abilities,
        pub storages: Vec<StorageInfo>,
    }

    #[derive(Debug, PartialEq)]
    pub enum CameraCommand {
        SetLiveView(bool),
        GetConfig,
        SetConfig(CameraSettings),
    }

    #[derive(Debug, PartialEq)]
    pub enum MessageToThread {
        ListCameras,
        CameraCommand(CameraCommand),
        UseCamera(String, String),
        CloseCamera,
        Break,
    }

    impl PartialEq for CameraInfo {
        fn eq(&self, _other: &Self) -> bool {
            // TODO
            true
        }
    }
}

pub struct CameraThread {
    handle: Option<JoinHandle<()>>,
    receiver: Receiver<messages::MessageFromThread>,
    sender: Sender<messages::MessageToThread>,
}

impl CameraThread {
    pub fn start() -> Self {
        let (to_thread_send, to_thread_recv) = std::sync::mpsc::channel();
        let (from_thread_send, from_thread_recv) = std::sync::mpsc::channel();

        let handle = std::thread::spawn(move || {
            camera_loop::camera_loop(to_thread_recv, from_thread_send)
                .expect("Failed to start camera loop"); // TODO handle this error somehow
        });

        Self {
            handle: Some(handle),
            receiver: from_thread_recv,
            sender: to_thread_send,
        }
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.sender.send(messages::MessageToThread::Break)?;

        if let Some(join_handle) = self.handle.take() {
            join_handle.join().unwrap()
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
}
