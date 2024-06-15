use std::error::Error;

// por ahora en el cliente, despues va a pasar al servidor
mod machine;

// TODO: sacar esto despues
#[allow(dead_code)]
pub struct CommandResolver {
    hostname: String,
    servicename: String,
    m: machine::Machine,
}

impl CommandResolver {
    pub fn new(hostname: String, servicename: String) -> Result<CommandResolver, Box<dyn Error>> {
        dbg!(&hostname);
        dbg!(&servicename);
        Ok(CommandResolver {
            hostname,
            servicename,
            m: machine::Machine::with(700).unwrap(),
        })
    }

    pub fn insert_coin(&mut self) -> Result<u32, Box<dyn Error>> {
        Ok(self.m.insert_coin())
    }

    pub fn consult_pool(&self) -> Result<u32, Box<dyn Error>> {
        Ok(self.m.get_pool())
    }

    pub fn leave(&self) {
        println!("Desconectandose del servidor...");
    }
}
