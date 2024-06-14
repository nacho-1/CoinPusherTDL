use crate::server::network_connection::NetworkConnection;
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::time::Duration;
use std::{io, thread};
use std::collections::HashMap;
use std::io::{Read, Write};

use crate::server::server_controller::ServerController;
use crate::server::server_error::{ServerError, ServerErrorKind};

use crate::server::traits::Config;
use thread_joiner::ThreadJoiner;
use threadpool::ThreadPool;

mod network_connection;
mod server_controller;
mod server_error;
mod traits;

pub type ServerResult<T> = Result<T, ServerError>;
pub type ClientId = usize;

const CONNECTION_WAIT_TIMEOUT: Duration = Duration::from_secs(180);
const ACCEPT_SLEEP_DUR: Duration = Duration::from_millis(100);

pub struct Server<C: Config> {
    pool: Mutex<ThreadPool>,
    config: C,
    clients: RwLock<HashMap<ClientId, TcpStream>>
}

impl<C: Config> Server<C> {
    pub fn new(config: C, threadpool_size: usize) -> Arc<Server<C>> {
        Arc::new(Server {
            pool: Mutex::new(ThreadPool::new(threadpool_size)),
            config,
            clients: RwLock::new(HashMap::new())
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
        thread_joiner.spawn(move || {
            sv_copy._run_client(network_connection).unwrap_or_else(|e| {
                if e.kind() != ServerErrorKind::ClientDisconnected
                {
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
        match self.connect_client(&mut network_connection) {
            Ok(connect_info) => {
                self.manage_successful_connection(connect_info, network_connection)?
            }
            Err(err) => self.manage_failed_connection(network_connection, err)?,
        };
        Ok(())
    }

    fn connect_client(
        self: &Arc<Self>,
        network_connection: &mut NetworkConnection<TcpStream, SocketAddr>,
    ) -> ServerResult<ClientId> {
        println!("Client connecting");
        let mut clients = self.clients.write()?;
        let client_id = clients.len();
        clients.insert(client_id, network_connection.stream().try_clone()?);
        Ok(client_id)
    }

    fn manage_successful_connection(
        self: &Arc<Self>,
        connect_info: ClientId,
        mut network_connection: NetworkConnection<TcpStream, SocketAddr>,
    ) -> ServerResult<()> {
        println!("Accepted client with ID {}", connect_info);
        // Returning the ID as a confirmation
        network_connection.write_all(connect_info.to_string().as_bytes())?;
        self.client_loop(&connect_info, &mut network_connection).unwrap_or(false);
        Ok(())
    }

    fn client_loop(
        self: &Arc<Self>,
        client_id: &ClientId,
        network_connection: &mut NetworkConnection<TcpStream, SocketAddr>,
    ) -> ServerResult<bool> {

        loop {
            match self.process_packet(network_connection, client_id) {
                // TODO: keep the loop going according to the received packet
                Ok(true) => {
                    continue;
                }
                Err(err) => {
                    if err.kind() != ServerErrorKind::ClientDisconnected {
                        eprintln!("Unexpected error: {}", err);
                    }
                    return Ok(false);
                }
            }
        }
    }

    // TODO: uncomment this when implementing PacketType and process_packet_given_control_byte function is completed
    // pub fn process_packet<T: Read>(
    //     self: &Arc<Self>,
    //     stream: &mut T,
    //     client_id: &ClientId,
    // ) -> ServerResult<PacketType> {
    //     let mut control_byte_buff = [0u8; 1];
    //     stream.read_exact(&mut control_byte_buff)?;
    //     self.process_packet_given_control_byte(control_byte_buff[0], stream, client_id)
    // }

    // TODO: implement PacketType, the idea is that the server will be able to process different types of packets (actions)
    // fn process_packet_given_control_byte<T: Read>(
    //     self: &Arc<Self>,
    //     control_byte: u8,
    //     stream: &mut T,
    //     id: &ClientId,
    // ) -> ServerResult<PacketType> {
    //     let packet_type = PacketType::try_from(control_byte)?;
    //     match packet_type {
    //         PacketType::JoinGame => {
    //             let publish = JoinGame::read_from(stream, control_byte)?;
    //             self.to_threadpool(|server, id| server.handle_join_game(publish, id), id)?;
    //         }
    //         PacketType::AddCoin => {
    //             let packet = AddCoin::read_from(stream, control_byte)?;
    //             self.to_threadpool(|server, id| server.handle_add_coin(packet, id), id)?;
    //         }
    //         PacketType::Disconnect => {
    //             let _packet = Disconnect::read_from(stream, control_byte)?;
    //         }
    //         _ => {
    //             return Err(ServerError::new_kind(
    //                 "Unexpected packet",
    //                 ServerErrorKind::ProtocolViolation,
    //             ))
    //         }
    //     }
    //     println!("Processing {}", packet_type);
    //     Ok(packet_type)
    // }

    fn to_threadpool<F>(self: &Arc<Self>, action: F, id: &ClientId) -> ServerResult<()>
        where
            F: FnOnce(Arc<Self>, &ClientId) -> ServerResult<()> + Send + 'static,
    {
        let sv_copy = self.clone();
        let id_copy = id.to_owned();
        self.pool.lock()?.execute(move || {
            action(sv_copy, &id_copy).unwrap_or_else(|e| {
                if e.kind() != ServerErrorKind::ClientNotFound
                    && e.kind() != ServerErrorKind::ClientDisconnected
                {
                    eprintln!("{}", e);
                }
            });
        })?;
        Ok(())
    }




}
