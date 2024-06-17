use std::error::Error;
use std::net::TcpStream;

use common::protocol::{
    StreamToServer,
    ClientMessage,
    ServerMessage,
};

// TODO: sacar esto despues
//#[allow(dead_code)]
pub struct CommandResolver {
    stream: StreamToServer,
}

impl CommandResolver {
    pub fn new(hostname: String, servicename: String) -> Result<CommandResolver, Box<dyn Error>> {
        dbg!(&hostname);
        dbg!(&servicename);
        let mut addr = hostname.clone();
        addr.push_str(":");
        addr.push_str(&servicename);

        let tcpstream = TcpStream::connect(addr)?;
        let stream = StreamToServer::new(tcpstream);
        Ok(CommandResolver {
            stream,
        })
    }

    pub fn insert_coin(&mut self) -> Result<u32, Box<dyn Error>> {
        self.stream.send_message(ClientMessage::Insert)?;

        let response = self.stream.recv_message();

        match response {
            Ok(ServerMessage::FellCoins(n)) => Ok(n),
            Ok(_) => panic!("Invalid server response"),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub fn consult_pool(&mut self) -> Result<u32, Box<dyn Error>> {
        self.stream.send_message(ClientMessage::ConsultPool)?;

        let response = self.stream.recv_message();

        match response {
            Ok(ServerMessage::PoolState(n)) => Ok(n),
            Ok(_) => panic!("Invalid server response"),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub fn leave(&mut self) {
        println!("Desconectandose del servidor...");

        let _ = self.stream.send_message(ClientMessage::Quit);
    }
}
