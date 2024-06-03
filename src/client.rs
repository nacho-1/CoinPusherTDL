use std::error::Error;
use std::io;

mod command_resolver;
use command_resolver::CommandResolver;

const INSERT_KEY: char = 't';
const ASK_KEY: char = 'y';
const QUIT_KEY: char = 'q';

pub struct ClientConfig {
    hostname: String,
    servicename: String,
}

impl ClientConfig {
    pub fn build(
        mut args: impl Iterator<Item = String>,
    ) -> Result<ClientConfig, &'static str> {
        // skip first arg
        args.next();

        let hostname = match args.next() {
            Some(arg) => arg,
            None => return Err("No se obtuvo la dirección del servidor"),
        };

        let servicename = match args.next() {
            Some(arg) => arg,
            None => return Err("No se obtuvo el puerto del servidor"),
        };

        Ok( ClientConfig {
                hostname,
                servicename,
            }
        )
    }
}

pub fn run(config: ClientConfig) -> Result<(), Box<dyn Error>> {
    let mut resolver = CommandResolver::new(config.hostname, config.servicename)?;

    loop {
        let option = read_option()?;
        match option {
            QUIT_KEY => return handle_quit(&resolver),
            INSERT_KEY => handle_insert(&mut resolver)?,
            ASK_KEY => handle_ask(&resolver)?,
            other => println!("[{other}] no es una opción válida\n"),
        }
    }
}

fn read_option() -> Result<char, Box<dyn Error>> {
    loop {
        println!("Ingrese una accion:");
        println!(" {INSERT_KEY} : Ingresar moneda");
        println!(" {ASK_KEY} : Consultar cantidad");
        println!(" {QUIT_KEY} : Salir");

        let mut input = String::new();

        io::stdin().read_line(&mut input)?;

        let trimmed_input = input.trim();

        if trimmed_input.len() != 1 {
            println!("Ingrese solamente un caracter\n");
            continue;
        }

        // Should never panic
        let option = trimmed_input.chars().next().unwrap();
        return Ok(option);
    }
}

fn handle_quit(resolver: &CommandResolver) -> Result<(), Box<dyn Error>> {
    resolver.leave();
    println!("Cerrando la aplicación...");
    Ok(())
}

fn handle_insert(resolver: &mut CommandResolver) -> Result<(), Box<dyn Error>> {
    let fell = resolver.insert_coin()?;

    if fell == 0 {
        println!("No cayeron monedas. Mala suerte.\n");
    } else {
        println!("Felicidades! Ganaste {fell} monedas!\n");
    }

    Ok(())
}

fn handle_ask(resolver: &CommandResolver) -> Result<(), Box<dyn Error>> {
    let pool = resolver.consult_pool()?;

    println!("Hay {pool} monedas en la maquina\n");

    Ok(())
}
