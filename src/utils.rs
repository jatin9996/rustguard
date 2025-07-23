use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use rand::{distributions::Alphanumeric, Rng};

// Validate if a string is a valid IP address
pub fn is_valid_ip(ip: &str) -> bool {
    IpAddr::from_str(ip).is_ok()
}

// Validate if a port is in the valid range (1-65535)
pub fn is_valid_port(port: u16) -> bool {
    port > 0 && port <= 65535
}

// Parse a host:port string into a SocketAddr
pub fn parse_host_port(s: &str) -> Option<SocketAddr> {
    SocketAddr::from_str(s).ok()
}

// Simple logging helper (prints with a tag)
pub fn log_with_tag(tag: &str, message: &str) {
    println!("[{}] {}", tag, message);
}

// Error formatting helper
pub fn format_error<E: std::fmt::Display>(err: E) -> String {
    format!("Error: {}", err)
}

// Generate a random alphanumeric ID of given length
pub fn generate_random_id(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
} 