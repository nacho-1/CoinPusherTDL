use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
};

pub struct ServerController {
    shutdown_bool: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl ServerController {
    pub fn new(shutdown_bool: Arc<AtomicBool>, handle: JoinHandle<()>) -> ServerController {
        ServerController {
            shutdown_bool,
            handle: Some(handle),
        }
    }
}

impl Drop for ServerController {
    fn drop(&mut self) {
        self.shutdown_bool.store(true, Ordering::Relaxed);
        let handle = self
            .handle
            .take()
            .expect("Server tried to shut down but it was already turned off");
        let id = handle.thread().id();
        if let Err(e) = handle.join() {
            eprintln!("{:?}: Thread joined with panic: {:?}", id, e);
        } else {
            eprintln!("{:?}: Server thread joined successfully", id);
        }
    }
}
