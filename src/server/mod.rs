use std::{io, thread};
use std::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::Duration;
use crate::server::network_connection::NetworkConnection;

use crate::server::server_controller::ServerController;
use crate::server::server_error::{ServerError, ServerErrorKind};

use thread_joiner::ThreadJoiner;
use threadpool::ThreadPool;
use crate::server::traits::Config;

mod server_controller;
mod network_connection;
mod server_error;
mod traits;

pub type ServerResult<T> = Result<T, ServerError>;

const CONNECTION_WAIT_TIMEOUT: Duration = Duration::from_secs(180);
const ACCEPT_SLEEP_DUR: Duration = Duration::from_millis(100);

pub struct Server<C: Config>  {
    pool: Mutex<ThreadPool>,
    config: C
}

impl<C: Config> Server<C> {
    pub fn new(config: C, threadpool_size: usize) -> Arc<Server<C>> {
        Arc::new(Server {
            pool: Mutex::new(ThreadPool::new(threadpool_size)),
            config
        })
    }

    pub fn run(self: Arc<Self>) -> io::Result<ServerController> {
        let shutdown_bool = Arc::new(AtomicBool::new(false));
        let shutdown_bool_copy = shutdown_bool.clone();
        let (started_sender, started_receiver) = mpsc::channel();

        let server_handle = thread::Builder::new()
            .name("server_loop".to_owned())
            .spawn(move || {
                if let Err(err) = self.server_loop(shutdown_bool, started_sender) {
                    eprintln!(
                        "Unexpected server error: {} - Try shutting down the server and restarting it",
                        err.to_string()
                    )
                }
            })?;
        println!("Creating new thread: {:?}", server_handle.thread().id());
        started_receiver.recv().unwrap_or_else(|e| {
            eprintln!("Error starting up server: {}", e);
        });
        let server_controller = ServerController::new(shutdown_bool_copy, server_handle);
        Ok(server_controller)
    }

    fn accept_client(
        self: &Arc<Self>,
        listener: &TcpListener,
    ) -> ServerResult<NetworkConnection<TcpStream, SocketAddr>> {
        match listener.accept() {
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                Err(ServerError::new_kind("Idle", ServerErrorKind::Idle))
            }
            Err(error) => {
                eprintln!("Error accepting TCP connection: {}", error);
                Err(ServerError::from(error))
            }
            Ok((stream, socket_addr)) => {
                stream.set_read_timeout(Some(CONNECTION_WAIT_TIMEOUT))?;
                Ok(NetworkConnection::new(socket_addr, stream))
            }
        }
    }

    fn server_loop(
        self: Arc<Self>,
        shutdown_bool: Arc<AtomicBool>,
        started_sender: Sender<()>,
    ) -> ServerResult<()> {
        let listener = TcpListener::bind(format!("{}:{}", self.config.ip(), self.config.port()))?;
        started_sender.send(())?;

        let mut thread_joiner = ThreadJoiner::new();
        listener.set_nonblocking(true)?;
        while !shutdown_bool.load(Ordering::Relaxed) {
            match self.accept_client(&listener) {
                Ok(connection_stream) => {
                    let socket_addr = *connection_stream.id();
                    self.run_client(connection_stream, &mut thread_joiner)
                        .unwrap_or_else(|e| eprintln!("{}: Error - {}", socket_addr, e));
                }
                Err(e) if e.kind() == ServerErrorKind::Idle => {
                    thread::sleep(ACCEPT_SLEEP_DUR);
                }
                Err(e) => {
                    eprintln!("New connection error: {}", e);
                    break;
                }
            }
        }
        self.shutdown()
    }

    fn run_client(
        self: &Arc<Self>,
        network_connection: NetworkConnection<TcpStream, SocketAddr>,
        thread_joiner: &mut ThreadJoiner,
    ) -> ServerResult<()> {
        let sv_copy = self.clone();
        // thread_joiner.spawn(move || {
        //     sv_copy._run_client(network_connection).unwrap_or_else(|e| {
        //         if e.kind() != ServerErrorKind::ClientDisconnected
        //         {
        //             eprintln!("Unhandled error {}", e);
        //         }
        //     });
        // });
        Ok(())
    }

    fn shutdown(self: &Arc<Self>) -> ServerResult<()> {
        println!("Shutting down server");
        // TODO: implement shutdown
        Ok(())
    }

    // fn _run_client(
    //     self: Arc<Self>,
    //     mut network_connection: NetworkConnection<TcpStream, SocketAddr>,
    // ) -> ServerResult<()> {
    //     match self.connect_client(&mut network_connection) {
    //         Ok(connect_info) => {
    //             self.manage_successful_connection(connect_info, network_connection)?
    //         }
    //         Err(err) => self.manage_failed_connection(network_connection, err)?,
    //     };
    //     Ok(())
    // }

    // fn connect_client(
    //     self: &Arc<Self>,
    //     network_connection: &mut NetworkConnection<TcpStream, SocketAddr>,
    // ) -> ServerResult<ConnectInfo> {
    //     println!("Client connecting");
    //     // TODO: implement saving client info
    //     let connect_info = self
    //         .clients_manager
    //         .write()?
    //         .new_session(network_connection.try_clone()?)?;
    //     Ok(connect_info)
    // }

}