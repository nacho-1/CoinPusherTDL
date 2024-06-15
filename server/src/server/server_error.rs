use std::{
    fmt, io,
    sync::{mpsc::SendError, PoisonError},
};

use common::thread_pool_error::ThreadPoolError;

#[derive(Debug)]
pub struct ServerError {
    msg: String,
    kind: ServerErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerErrorKind {
    ProtocolViolation,
    ClientDisconnected,
    ClientNotFound,
    Timeout,
    PoisonedLock,
    Irrecoverable,
    Idle,
    Other,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for ServerError {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl From<io::Error> for ServerError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::UnexpectedEof
            | io::ErrorKind::NotConnected
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::ConnectionReset => ServerError::new_kind(
                "Unexpected disconnection",
                ServerErrorKind::ClientDisconnected,
            ),
            io::ErrorKind::WouldBlock => {
                ServerError::new_kind("Connection timeout", ServerErrorKind::Timeout)
            }
            _ => ServerError::new_msg(format!("{:?}", error)),
        }
    }
}

impl<T> From<PoisonError<T>> for ServerError {
    fn from(err: PoisonError<T>) -> Self {
        ServerError::new_kind(err.to_string(), ServerErrorKind::PoisonedLock)
    }
}

impl From<SendError<()>> for ServerError {
    fn from(err: SendError<()>) -> Self {
        eprintln!("Sender error: {}", err);
        ServerError::new_msg(err.to_string())
    }
}

impl From<ThreadPoolError> for ServerError {
    fn from(err: ThreadPoolError) -> Self {
        eprintln!("ThreadPool error: {}", err);
        ServerError::new_kind(
            format!("ThreadPoolError: {}", err),
            ServerErrorKind::Irrecoverable,
        )
    }
}

impl ServerError {
    pub fn new_msg<T: Into<String>>(msg: T) -> ServerError {
        ServerError {
            msg: msg.into(),
            kind: ServerErrorKind::Other,
        }
    }

    pub fn new_kind<T: Into<String>>(msg: T, kind: ServerErrorKind) -> ServerError {
        ServerError {
            msg: msg.into(),
            kind,
        }
    }

    pub fn kind(&self) -> ServerErrorKind {
        self.kind
    }
}
