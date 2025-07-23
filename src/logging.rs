

pub fn log_info(msg: &str) {
    println!("[INFO] {}", msg);
}

pub fn log_error(msg: &str) {
    eprintln!("[ERROR] {}", msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_info_and_error() {
        log_info("info message");
        log_error("error message");
        // No assertion, just ensure no panic
    }
} 