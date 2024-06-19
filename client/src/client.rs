use std::error::Error;
use std::io;

mod command_resolver;
use command_resolver::CommandResolver;

const INSERT_KEY: char = 't';
const ASK_KEY: char = 'y';
const QUIT_KEY: char = 'q';

/// Procesador de argumentos del cliente
pub struct ClientConfig {
    hostname: String,
    servicename: String,
}

impl ClientConfig {
    /// Crea la instancia.
    /// Se asume que el primer argumento es el path del ejecutable.
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<ClientConfig, &'static str> {
        // skip first arg
        args.next();

        let hostname = match args.next() {
            Some(arg) => arg,
            None => return Err("Could not get the hostname of the server"),
        };

        let servicename = match args.next() {
            Some(arg) => arg,
            None => return Err("Could not get the servicename of the server"),
        };

        Ok(ClientConfig {
            hostname,
            servicename,
        })
    }
}

pub fn run(config: ClientConfig) -> Result<(), Box<dyn Error>> {
    let mut resolver = CommandResolver::new(config.hostname, config.servicename)?;

    loop {
        let option = read_option()?;
        match option {
            QUIT_KEY => return handle_quit(&mut resolver),
            INSERT_KEY => handle_insert(&mut resolver)?,
            ASK_KEY => handle_ask(&mut resolver)?,
            other => println!("[{other}] is not a valid option\n"),
        }
    }
}

fn read_option() -> Result<char, Box<dyn Error>> {
    loop {
        println!("Choose an action:");
        println!(" {INSERT_KEY} : Insert coin");
        println!(" {ASK_KEY} : Check coins");
        println!(" {QUIT_KEY} : Quit");

        let mut input = String::new();

        io::stdin().read_line(&mut input)?;

        let trimmed_input = input.trim();

        if trimmed_input.len() != 1 {
            println!("Only one character is allowed\n");
            continue;
        }

        // Should never panic
        let option = trimmed_input.chars().next().unwrap();
        return Ok(option);
    }
}

fn handle_quit(resolver: &mut CommandResolver) -> Result<(), Box<dyn Error>> {
    resolver.leave();
    println!("Closing the application...");
    Ok(())
}

fn handle_insert(resolver: &mut CommandResolver) -> Result<(), Box<dyn Error>> {
    let fell = resolver.insert_coin()?;

    if fell == 0 {
        println!("No coins fell. Bad luck.\n");
    } else {
        println!("Congrats! You won {fell} coins!\n");
    }

    Ok(())
}

fn handle_ask(resolver: &mut CommandResolver) -> Result<(), Box<dyn Error>> {
    let pool = resolver.consult_pool()?;

    println!("There are {pool} coins in the machine\n");

    Ok(())
}
