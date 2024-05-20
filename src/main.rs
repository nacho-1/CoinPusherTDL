use std::process;

//mod server;
mod client;

fn main() {
    if let Err(e) = client::run() {
        eprintln!("Error corriendo la aplicación: {e}");
        process::exit(1);
    }
}

