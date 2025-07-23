// Configuration structs and loading logic

pub struct Config {
    pub icap_server_addr: String,
    pub listen_addr: String,
    pub fallback_mode: bool,
}

impl Config {
    pub fn load() -> Self {
        // Load config from file, env, or defaults
        Config {
            icap_server_addr: "127.0.0.1:1344".to_string(),
            listen_addr: "0.0.0.0:8080".to_string(),
            fallback_mode: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load() {
        let config = Config::load();
        assert_eq!(config.icap_server_addr, "127.0.0.1:1344");
        assert_eq!(config.listen_addr, "0.0.0.0:8080");
        assert!(!config.fallback_mode);
    }
} 