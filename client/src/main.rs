use std::env;
use std::process;

mod client;
use client::ClientConfig;

fn main() {
    let config = ClientConfig::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Error while reading arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = client::run(config) {
        eprintln!("Error while running the application: {e}");
        process::exit(1);
    }
}
