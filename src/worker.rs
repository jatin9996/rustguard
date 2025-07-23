// ZeroMQ worker: Receives content, inspects, and returns result.

use zmq;

pub struct Worker {
    pub bind_addr: String,
}

impl Worker {
    pub fn new(bind_addr: String) -> Self {
        Worker { bind_addr }
    }

    pub fn run(&self) {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::REP).expect("Failed to create REP socket");
        socket.bind(&self.bind_addr).expect("Failed to bind REP socket");
        println!("Worker listening on {}", self.bind_addr);
        loop {
            let msg = match socket.recv_msg(0) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Worker recv error: {}", e);
                    continue;
                }
            };
            let content = msg.as_str().unwrap_or("");
            // Content inspection: block if contains malware or phishing, else OK, else MODIFIED
            let reply = if content.contains("malware") || content.contains("phishing") {
                b"BLOCKED".to_vec()
            } else if content.contains("editme") {
                // Simulate modification
                content.replace("editme", "[MODIFIED]").into_bytes()
            } else {
                msg.to_vec()
            };
            socket.send(reply, 0).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_new() {
        let worker = Worker::new("tcp://*:5555".to_string());
        assert_eq!(worker.bind_addr, "tcp://*:5555");
    }

    // #[test]
    // fn test_run() {
    //     // TODO: Mock zmq context/socket, test run logic
    // }
} 