use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle, ThreadId},
};

pub struct ThreadJoiner {
    finished_sender: Sender<Message>,
    joiner_thread_handle: Option<JoinHandle<()>>,
}

pub struct ThreadGuard {
    id: ThreadId,
    sender: Sender<Message>,
}

enum Message {
    Started(JoinHandle<()>),
    Finished(ThreadId),
    Stop,
}

impl Default for ThreadJoiner {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreadJoiner {
    pub fn new() -> ThreadJoiner {
        let (sender, receiver) = mpsc::channel();
        let joiner_thread_handle = thread::spawn(move || {
            ThreadJoiner::join_loop(receiver);
        });

        ThreadJoiner {
            finished_sender: sender,
            joiner_thread_handle: Some(joiner_thread_handle),
        }
    }

    pub fn spawn<F>(&mut self, action: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let sender_clone = self.finished_sender.clone();
        let handle = thread::spawn(move || {
            let guard = ThreadGuard::new(thread::current().id(), sender_clone);
            action();
            drop(guard);
        });
        self.finished_sender.send(Message::Started(handle)).unwrap();
    }

    /// Executes the loop that joins the threads
    fn join_loop(receiver: Receiver<Message>) {
        let mut handles = HashMap::new();
        for message in receiver {
            match message {
                Message::Started(handle) => {
                    handles.insert(handle.thread().id(), handle);
                }
                Message::Finished(id) => {
                    if let Some(handle) = handles.remove(&id) {
                        Self::join(id, handle);
                    }
                }
                Message::Stop => break,
            }
        }
        Self::join_remaining(handles);
    }

    fn join_remaining(handles: HashMap<ThreadId, JoinHandle<()>>) {
        for (id, handle) in handles {
            Self::join(id, handle);
        }
    }

    fn join(id: ThreadId, handle: JoinHandle<()>) {
        handle.join().unwrap_or_else(|e| {
            eprintln!("{:?} - Thread joined with panic: {:?}", id, e);
        });
    }
}

impl Drop for ThreadJoiner {
    fn drop(&mut self) {
        self.finished_sender
            .send(Message::Stop)
            .unwrap_or_else(|e| {
                eprintln!("Error de Sender: {}", e);
            });

        let joiner_thread_id = self.joiner_thread_handle.as_ref().unwrap().thread().id();
        println!(
            "Joining helper thread from ThreadJoiner{:?}",
            joiner_thread_id
        );
        self.joiner_thread_handle
            .take()
            .expect("Joiner thread handle is None")
            .join()
            .unwrap_or_else(|e| {
                eprintln!("{:?} - Thread joined with panic: {:?}", joiner_thread_id, e);
            });
    }
}

impl ThreadGuard {
    fn new(id: ThreadId, sender: Sender<Message>) -> Self {
        ThreadGuard { id, sender }
    }
}

impl Drop for ThreadGuard {
    fn drop(&mut self) {
        println!("ThreadGuard drop: {:?}", self.id);

        self.sender.send(Message::Finished(self.id)).unwrap_or(());
    }
}
