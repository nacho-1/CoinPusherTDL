use std::net::TcpStream;
use std::io::{Read};
use std::str;

pub struct Message {
    pub action: Action,
    pub value: String,
}

#[derive(Debug)]
pub enum Action {
    Insert,
    Quit,
    ConsultPool,
}

impl Action {
    fn to_char(&self) -> char {
        match self {
            Action::Insert => 't',
            Action::Quit => 'q',
            Action::ConsultPool => 'y',
        }
    }

    fn from_char(c: char) -> Option<Action> {
        match c {
            't' => Some(Action::Insert),
            'q' => Some(Action::Quit),
            'y' => Some(Action::ConsultPool),
            _ => None,
        }
    }
}

pub struct Protocol;

impl Protocol {
    pub fn encode(message: Message) -> Vec<u8> {
        let action_char = message.action.to_char();
        format!("{}{}", action_char, message.value).into_bytes()
    }
    
    pub fn decode(mut stream: TcpStream) -> Message {
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).expect("Error: Cannot read from stream");
        let input = str::from_utf8(&buffer).expect("Invalid input: Cannot convert bytes to UTF-8 string");

        if input.is_empty() { 
            panic!("Invalid input: String is empty");
        }

        let action_char = input.chars().next().unwrap();
        let action = Action::from_char(action_char).expect("Invalid input: The action provided is unknown");

        let value = input[1..].to_string();


        Message {
            action,
            value,
        }
    }
}