use std::error::Error;
use std::io;

mod command_resolver;

const INSERT_KEY: char = 't';
const ASK_KEY: char = 'y';
const QUIT_KEY: char = 'q';

pub fn run() -> Result<(), Box<dyn Error>> {
    loop {
        let option = read_option()?;
        match option {
            QUIT_KEY => return handle_quit(),
            INSERT_KEY => handle_insert()?,
            ASK_KEY => handle_ask()?,
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

fn handle_quit() -> Result<(), Box<dyn Error>> {
    command_resolver::leave();
    println!("Cerrando la aplicación...");
    Ok(())
}

fn handle_insert() -> Result<(), Box<dyn Error>> {
    let fell = command_resolver::insert_coin()?;

    if fell == 0 {
        println!("No cayeron monedas. Mala suerte.\n");
    } else {
        println!("Felicidades! Ganaste {fell} monedas!\n");
    }

    Ok(())
}

fn handle_ask() -> Result<(), Box<dyn Error>> {
    let pool = command_resolver::consult_pool()?;

    println!("Hay {pool} monedas en la maquina\n");

    Ok(())
}
