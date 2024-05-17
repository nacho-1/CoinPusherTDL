use std::io;
use std::process;
use std::error::Error;

const INSERT_KEY: char = 't';
const ASK_KEY: char = 'y';
const QUIT_KEY: char = 'q';

fn main() {
    let option = read_option().unwrap_or_else(|err| {
        eprintln!("Error leyendo la opción: {err}");
        process::exit(1);
    });

    println!("La opción elegida es: {option}");
}

fn read_option() -> Result<char, Box<dyn Error>> {
    loop {
        println!("Ingresá una accion:");
        println!(" {INSERT_KEY} : Ingresar moneda");
        println!(" {ASK_KEY} : Consultar cantidad");
        println!(" {QUIT_KEY} : Salir");

        let mut input = String::new();

        io::stdin().read_line(&mut input)?;

        let trimmed_input = input.trim();

        if trimmed_input.len() != 1 {
            println!("Opción invalida. Intente nuevamente\n");
            continue;
        }

        // Should never panic
        let option = trimmed_input.chars().next().unwrap();
        return Ok(option);
    }
}
