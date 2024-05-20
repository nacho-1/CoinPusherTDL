use std::error::Error;
use std::io;

const INSERT_KEY: char = 't';
const ASK_KEY: char = 'y';
const QUIT_KEY: char = 'q';

pub fn run() -> Result<(), Box<dyn Error>> {
    loop {
        let option = read_option()?;
        match option {
            QUIT_KEY => return Ok(()),
            INSERT_KEY => println!("Insertó una moneda\n"),
            ASK_KEY => println!("Hay N monedas\n"),
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

