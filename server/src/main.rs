use crate::config::FileConfig;
use crate::server::Server;
use std::env;
use std::io::Read;

mod config;
mod server;

fn get_config_path(default_path: Option<String>) -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        return String::from(&args[1]);
    }
    if let Some(path) = default_path {
        return path;
    }
    panic!("Error: should provide a config file path")
}

pub fn init(config_path: &str) {
    let config = FileConfig::new(config_path).expect("Error while reading config file");

    let threadpool_size = 8;
    let server = Server::new(config, threadpool_size);
    let controller = server.run().expect("Error while running server");

    println!("Press [ENTER] to stop the server");

    let mut buf = [0u8; 1];
    std::io::stdin().read_exact(&mut buf).unwrap_or(());
    drop(controller);
}

fn main() {
    let config_path: String = get_config_path(Some("server/resources/config.txt".to_string()));
    init(&config_path);
}
