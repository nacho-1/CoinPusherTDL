use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::num::ParseIntError;
use std::str;

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
        let encoded_msg = encode_client_msg(msg);

        self.stream.write_all(&encoded_msg)?;

        Ok(())
    }

    pub fn recv_message(&mut self) -> Result<ServerMessage, ProtocolError> {
        //let mut buffer = Vec::<u8>::with_capacity(1);
        let mut buffer = vec![0u8; 1];

        self.stream.read_exact(&mut buffer)?;

        // Should never panic
        let msg_byte = char::from(buffer.pop().unwrap());

        match msg_byte {
            FELL_BYTE => {
                //let mut buffer = Vec::<u8>::with_capacity(5);
                let mut buffer = vec![0u8; 5];

                self.stream.read_exact(&mut buffer)?;

                let n = decode_count(&buffer)?;

                Ok(ServerMessage::FellCoins(n))
            }
            POOL_BYTE => {
                //let mut buffer = Vec::<u8>::with_capacity(5);
                let mut buffer = vec![0u8; 5];

                self.stream.read_exact(&mut buffer)?;

                let n = decode_count(&buffer)?;

                Ok(ServerMessage::PoolState(n))
            }
            c => {
                let msg = format!("Unknown server message: {}", c);
                Err(ProtocolError { msg })
            }
        }
    }
}

pub struct StreamToClient {
    stream: TcpStream,
}

impl StreamToClient {
    pub fn new(stream: TcpStream) -> Self {
        StreamToClient { stream }
    }

    pub fn send_message(&mut self, msg: ServerMessage) -> Result<(), ProtocolError> {
        let encoded_msg = encode_server_msg(msg)?;

        self.stream.write_all(&encoded_msg)?;
        Ok(())
    }

    pub fn recv_message(&mut self) -> Result<ClientMessage, ProtocolError> {
        //let mut buffer = Vec::<u8>::with_capacity(1);
        let mut buffer = vec![0u8; 1];

        self.stream.read_exact(&mut buffer)?;

        // Should never panic
        let msg_byte = char::from(buffer.pop().unwrap());

        match msg_byte {
            INSERT_BYTE => Ok(ClientMessage::Insert),
            CONSULT_BYTE => Ok(ClientMessage::ConsultPool),
            QUIT_BYTE => Ok(ClientMessage::Quit),
            c => {
                let msg = format!("Unknown client message: {}", c);
                Err(ProtocolError { msg })
            }
        }
    }
}

fn encode_client_msg(msg: ClientMessage) -> Vec<u8> {
    match msg {
        ClientMessage::Insert => format!("{}", INSERT_BYTE).into_bytes(),
        ClientMessage::ConsultPool => format!("{}", CONSULT_BYTE).into_bytes(),
        ClientMessage::Quit => format!("{}", QUIT_BYTE).into_bytes(),
    }
}

fn encode_server_msg(msg: ServerMessage) -> Result<Vec<u8>, ProtocolError> {
    match msg {
        ServerMessage::FellCoins(n) => {
            if n > 99999 {
                let msg = format!("n ({}) too big. Can't have more than 5 digits", n);
                Err(ProtocolError { msg })
            } else {
                Ok(format!("{}{:0>5}", FELL_BYTE, n.to_string()).into_bytes())
            }
        }
        ServerMessage::PoolState(n) => {
            if n > 99999 {
                let msg = format!("n ({}) too big. Can't have more than 5 digits", n);
                Err(ProtocolError { msg })
            } else {
                Ok(format!("{}{:0>5}", POOL_BYTE, n.to_string()).into_bytes())
            }
        }
    }
}

fn decode_count(buffer: &[u8]) -> Result<u32, ProtocolError> {
    // Should always be 5 bytes
    assert_eq!(buffer.len(), 5);
    let n = str::from_utf8(buffer)?.parse::<u32>()?;
    Ok(n)
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

//If all the exceptions are handled in the same way we can abstract it in order to prevent code repetition
//https://doc.rust-lang.org/book/ch17-02-trait-objects.html#using-trait-objects-that-allow-for-values-of-different-types
impl From<str::Utf8Error> for ProtocolError {
    fn from(err: str::Utf8Error) -> Self {
        ProtocolError {
            msg: format!("{}", err),
        }
    }
}

impl From<ParseIntError> for ProtocolError {
    fn from(err: ParseIntError) -> Self {
        ProtocolError {
            msg: format!("{}", err),
        }
    }
}

impl From<std::io::Error> for ProtocolError {
    fn from(err: std::io::Error) -> Self {
        ProtocolError {
            msg: format!("{}", err),
        }
    }
}

#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn encode_insert_msg() {
        let msg = ClientMessage::Insert;

        let encoded_msg = encode_client_msg(msg);
        let encoded_msg = str::from_utf8(&encoded_msg).unwrap();

        assert_eq!(encoded_msg, "t");
    }

    #[test]
    fn encode_consult_msg() {
        let msg = ClientMessage::ConsultPool;

        let encoded_msg = encode_client_msg(msg);
        let encoded_msg = str::from_utf8(&encoded_msg).unwrap();

        assert_eq!(encoded_msg, "y");
    }

    #[test]
    fn encode_quit_msg() {
        let msg = ClientMessage::Quit;

        let encoded_msg = encode_client_msg(msg);
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

    #[test]
    fn encode_fell_msg() {
        let msg = ServerMessage::FellCoins(0);

        let encoded_msg = encode_server_msg(msg).unwrap();
        let encoded_msg = str::from_utf8(&encoded_msg).unwrap();

        assert_eq!(encoded_msg, "f00000");
    }

    #[test]
    fn encode_pool_msg() {
        let msg = ServerMessage::PoolState(99999);

        let encoded_msg = encode_server_msg(msg).unwrap();
        let encoded_msg = str::from_utf8(&encoded_msg).unwrap();

        assert_eq!(encoded_msg, "p99999");
    }

    #[test]
    fn encode_invalid_fell_msg() {
        let msg = ServerMessage::FellCoins(100000);

        assert!(encode_server_msg(msg).is_err());
    }

    #[test]
    fn encode_invalid_pool_msg() {
        let msg = ServerMessage::PoolState(100000);

        assert!(encode_server_msg(msg).is_err());
    }
}
