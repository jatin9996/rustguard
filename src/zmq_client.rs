// ZeroMQ client: Forwards content to worker and receives scan results.

use zmq;

pub struct ZmqClient {
    pub addr: String,
}

impl Clone for ZmqClient {
    fn clone(&self) -> Self {
        ZmqClient { addr: self.addr.clone() }
    }
}

// Trait to abstract ZMQ socket for testability
pub trait ZmqSocket {
    fn send(&self, content: &[u8], flags: i32) -> Result<(), String>;
    fn recv_msg(&self, flags: i32) -> Result<Vec<u8>, String>;
}

// Real implementation for zmq::Socket
impl ZmqSocket for zmq::Socket {
    fn send(&self, content: &[u8], flags: i32) -> Result<(), String> {
        zmq::Socket::send(self, content, flags).map_err(|e| e.to_string())
    }
    fn recv_msg(&self, flags: i32) -> Result<Vec<u8>, String> {
        zmq::Socket::recv_msg(self, flags).map(|m| m.to_vec()).map_err(|e| e.to_string())
    }
}

impl ZmqClient {
    pub fn new(addr: String) -> Self {
        ZmqClient { addr }
    }

    pub fn send_and_receive(&self, content: &[u8]) -> Result<Vec<u8>, String> {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::REQ).map_err(|e| e.to_string())?;
        socket.connect(&self.addr).map_err(|e| e.to_string())?;
        Self::send_and_receive_with_socket(&socket, content)
    }

    // New: Accepts any ZmqSocket for testability
    pub fn send_and_receive_with_socket<S: ZmqSocket>(socket: &S, content: &[u8]) -> Result<Vec<u8>, String> {
        socket.send(content, 0)?;
        socket.recv_msg(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zmq_client_new() {
        let client = ZmqClient::new("tcp://127.0.0.1:5555".to_string());
        assert_eq!(client.addr, "tcp://127.0.0.1:5555");
    }

    // Mock socket for testing
    struct MockSocket {
        pub sent: std::cell::RefCell<Vec<u8>>,
        pub to_receive: Vec<u8>,
    }
    impl ZmqSocket for MockSocket {
        fn send(&self, content: &[u8], _flags: i32) -> Result<(), String> {
            self.sent.replace(content.to_vec());
            Ok(())
        }
        fn recv_msg(&self, _flags: i32) -> Result<Vec<u8>, String> {
            Ok(self.to_receive.clone())
        }
    }

    #[test]
    fn test_send_and_receive() {
        let mock = MockSocket {
            sent: std::cell::RefCell::new(vec![]),
            to_receive: b"scan_result".to_vec(),
        };
        let content = b"test_content";
        let result = ZmqClient::send_and_receive_with_socket(&mock, content).unwrap();
        assert_eq!(result, b"scan_result");
        assert_eq!(&*mock.sent.borrow(), content);
    }
} 