use std::process;
use std::env;

mod client;
use client::ClientConfig;

fn main() {
    let config = ClientConfig::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problema leyendo argumentos: {err}");
        process::exit(1);
    });

    if let Err(e) = client::run(config) {
        eprintln!("Error corriendo la aplicación: {e}");
        process::exit(1);
    }
}
