use std::net::TcpStream;
use std::io::{Read, Write};
use std::str;
//use std::error::Error;
use std::fmt;

const INSERT_BYTE: char = 't';
const CONSULT_BYTE: char = 'y';
const QUIT_BYTE: char = 'q';

const FELL_BYTE: char = 'f';
const POOL_BYTE: char = 'p';

#[derive(Debug)]
pub enum ClientMessage {
    Insert,
    ConsultPool,
    Quit,
}

#[derive(Debug)]
pub enum ServerMessage {
    FellCoins(u32),
    PoolState(u32),
}

pub struct StreamToServer {
    stream: TcpStream,
}

impl StreamToServer {
    pub fn new(stream: TcpStream) -> Self {
        StreamToServer { stream }
    }

    pub fn send_message(&mut self, msg: ClientMessage) -> Result<(), ProtocolError> {
        let encoded_msg = encode(msg);
        match self.stream.write_all(&encoded_msg) {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = format!("{}", e);
                Err( ProtocolError { msg } )
            },
        }
    }

    pub fn recv_message(&mut self) -> Result<ServerMessage, ProtocolError> {
        let mut buffer = Vec::<u8>::with_capacity(1);

        if let Err(e) = self.stream.read_exact(&mut buffer) {
            let msg = format!("{}", e);
            return Err( ProtocolError { msg } );
        }

        // Should never panic
        let msg_byte = char::from(buffer.pop().unwrap());

        match msg_byte {
            FELL_BYTE => {
                let mut buffer = Vec::<u8>::with_capacity(5);

                if let Err(e) = self.stream.read_exact(&mut buffer) {
                    let msg = format!("{}", e);
                    return Err( ProtocolError { msg } );
                }

                let n = decode_count(&buffer)?;

                Ok(ServerMessage::FellCoins(n))
            },
            POOL_BYTE => {
                let mut buffer = Vec::<u8>::with_capacity(5);

                if let Err(e) = self.stream.read_exact(&mut buffer) {
                    let msg = format!("{}", e);
                    return Err( ProtocolError { msg } );
                }

                let n = decode_count(&buffer)?;

                Ok(ServerMessage::PoolState(n))
            },
            c => {
                let msg = format!("Unknown server message: {}", c);
                Err( ProtocolError { msg } )
            }
        }
    }
}

fn encode(msg: ClientMessage) -> Vec::<u8> {
    match msg {
        ClientMessage::Insert => {
            format!("{}", INSERT_BYTE).into_bytes()
        },
        ClientMessage::ConsultPool => {
            format!("{}", CONSULT_BYTE).into_bytes()
        },
        ClientMessage::Quit => {
            format!("{}", QUIT_BYTE).into_bytes()
        },
    }
}

fn decode_count(buffer: &[u8]) -> Result<u32, ProtocolError> {
    // Should always be 5 bytes
    assert_eq!(buffer.len(), 5);
    // Seguro hay alguna forma de hacer esto mas lindo
    match str::from_utf8(buffer) {
        Ok(s) => {
            match s.parse::<u32>() {
                Ok(n) => Ok(n),
                Err(e) => {
                    let msg = format!("{}", e);
                    Err( ProtocolError { msg } )
                },
            }
        },
        Err(e) => {
            let msg = format!("{}", e);
            Err( ProtocolError { msg } )
        },
    }
}

#[derive(Debug)]
pub struct ProtocolError {
    msg: String,
}

impl std::error::Error for ProtocolError {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn encode_insert_msg() {
        let msg = ClientMessage::Insert;

        let encoded_msg = encode(msg);
        let encoded_msg = str::from_utf8(&encoded_msg).unwrap();

        assert_eq!(encoded_msg, "t");
    }

    #[test]
    fn encode_consult_msg() {
        let msg = ClientMessage::ConsultPool;

        let encoded_msg = encode(msg);
        let encoded_msg = str::from_utf8(&encoded_msg).unwrap();

        assert_eq!(encoded_msg, "y");
    }

    #[test]
    fn encode_quit_msg() {
        let msg = ClientMessage::Quit;

        let encoded_msg = encode(msg);
        let encoded_msg = str::from_utf8(&encoded_msg).unwrap();

        assert_eq!(encoded_msg, "q");
    }

    #[test]
    fn decode_counts() {
        let count_0 = "00000".as_bytes();
        let count_1 = "00001".as_bytes();
        let count_max = "99999".as_bytes();

        assert_eq!(decode_count(count_0).unwrap(), 0);
        assert_eq!(decode_count(count_1).unwrap(), 1);
        assert_eq!(decode_count(count_max).unwrap(), 99999);
    }

    #[test]
    fn decode_non_numeric_slice() {
        let s = "abcde".as_bytes();

        assert!(decode_count(s).is_err());
    }
}
