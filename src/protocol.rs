pub struct Message {
    pub action: Action,
    pub value: String,
}

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
    
    pub fn decode(bytes: Vec<u8>) -> Message {
        let input = std::str::from_utf8(&bytes).expect("Invalid input: Cannot convert bytes to UTF-8 string");

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