use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
};

use crate::server::traits::Config;

#[derive(Debug, Clone)]
pub struct FileConfig {
    port: u16,
    host: String,
}

const PORT_KEY: &str = "port";
const HOST_KEY: &str = "host";

const SEPARATOR: &str = "=";

impl FileConfig {
    pub fn new(path: &str) -> Option<FileConfig> {
        let config_file = File::open(path).ok()?;
        FileConfig::new_from_file(config_file)
    }

    pub fn new_from_file(config_file: impl Read) -> Option<FileConfig> {
        let lines: Vec<String> = BufReader::new(config_file)
            .lines()
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        let mut config: HashMap<String, String> = lines
            .iter()
            .map(|line| {
                let (key, value) = line.trim().split_once(SEPARATOR)?;
                Some((key.to_string(), value.to_string()))
            })
            .collect::<Option<HashMap<_, _>>>()?;

        Some(FileConfig {
            port: config.remove(PORT_KEY)?.parse().ok()?,
            host: config.remove(HOST_KEY)?,
        })
    }
}

impl Config for FileConfig {
    fn port(&self) -> u16 {
        self.port
    }

    fn host(&self) -> &str {
        &self.host
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::config::FileConfig;
    use crate::server::traits::Config;

    #[test]
    fn test_valid_file() {
        let cursor = Cursor::new(
            "port=8080
                    host=localhost",
        );

        let config = FileConfig::new_from_file(cursor).unwrap();
        assert_eq!(config.port(), 8080);
        assert_eq!(config.host(), "localhost");
    }

    #[test]
    fn test_valid_file_with_whitespace() {
        let cursor = Cursor::new(
            "port=8080
                    host=localhost",
        );

        let config = FileConfig::new_from_file(cursor).unwrap();
        assert_eq!(config.port(), 8080);
        assert_eq!(config.host(), "localhost");
    }

    #[test]
    fn test_invalid_key() {
        let cursor = Cursor::new(
            "invalid_key=8080
host=localhost",
        );

        assert!(FileConfig::new_from_file(cursor).is_none());
    }

    #[test]
    fn test_invalid_value() {
        let cursor = Cursor::new(
            "port=WWWW
host=localhost",
        );

        assert!(FileConfig::new_from_file(cursor).is_none());
    }
}
