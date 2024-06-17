use crate::server::network_connection::NetworkConnection;
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::{io, thread};

use crate::server::server_controller::ServerController;
use crate::server::server_error::{ServerError, ServerErrorKind};

use crate::machine::Machine;
use crate::server::traits::Config;
use common::protocol::{ClientMessage, ServerMessage, StreamToClient};
use thread_joiner::ThreadJoiner;

mod network_connection;
mod server_controller;
mod server_error;
pub(crate) mod traits;

pub type ServerResult<T> = Result<T, ServerError>;

const CONNECTION_WAIT_TIMEOUT: Duration = Duration::from_secs(180);
const ACCEPT_SLEEP_DUR: Duration = Duration::from_millis(100);

pub struct Server<C: Config> {
    config: C,
    coin_machine: Mutex<Machine>,
}

impl<C: Config> Server<C> {
    pub fn new(config: C) -> Arc<Server<C>> {
        Arc::new(Server {
            config,
            coin_machine: Mutex::new(Machine::with(200).unwrap()),
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
                        err
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
        let listener = TcpListener::bind(format!("{}:{}", self.config.host(), self.config.port()))?;
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
        thread_joiner.spawn(move || {
            sv_copy._run_client(network_connection).unwrap_or_else(|e| {
                if e.kind() != ServerErrorKind::ClientDisconnected {
                    eprintln!("Unhandled error {}", e);
                }
            });
        });
        Ok(())
    }

    fn shutdown(self: &Arc<Self>) -> ServerResult<()> {
        println!("Shutting down server");
        // TODO: implement shutdown
        Ok(())
    }

    fn _run_client(
        self: Arc<Self>,
        mut network_connection: NetworkConnection<TcpStream, SocketAddr>,
    ) -> ServerResult<()> {
        self.client_loop(&mut network_connection).unwrap_or(false);
        Ok(())
    }

    fn client_loop(
        self: &Arc<Self>,
        network_connection: &mut NetworkConnection<TcpStream, SocketAddr>,
    ) -> ServerResult<bool> {
        let mut stream_to_client = StreamToClient::new(network_connection.stream().try_clone()?);
        loop {
            match stream_to_client.recv_message() {
                Ok(client_message) => {
                    let response = self.process_message(client_message);
                    match response {
                        Some(response) => stream_to_client
                            .send_message(response)
                            .expect("Protocol error"),
                        _ => return Ok(true),
                    }
                }
                Err(err) => {
                    eprintln!("Unexpected error: {}", err);
                }
            }
        }
    }

    fn process_message(self: &Arc<Self>, client_message: ClientMessage) -> Option<ServerMessage> {
        match client_message {
            ClientMessage::Insert => {
                let fell_coins = self.coin_machine.lock().ok()?.insert_coin();
                Some(ServerMessage::FellCoins(fell_coins))
            }
            ClientMessage::ConsultPool => {
                let coins = self.coin_machine.lock().ok()?.get_pool();
                Some(ServerMessage::PoolState(coins))
            }
            ClientMessage::Quit => None,
        }
    }
}
