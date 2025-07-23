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

impl ZmqClient {
    pub fn new(addr: String) -> Self {
        ZmqClient { addr }
    }

    pub fn send_and_receive(&self, content: &[u8]) -> Result<Vec<u8>, String> {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::REQ).map_err(|e| e.to_string())?;
        socket.connect(&self.addr).map_err(|e| e.to_string())?;
        socket.send(content, 0).map_err(|e| e.to_string())?;
        let msg = socket.recv_msg(0).map_err(|e| e.to_string())?;
        Ok(msg.to_vec())
    }
} 