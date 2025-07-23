

use std::net::SocketAddr;
use std::io::{Read, Write};
use std::net::TcpListener;
use crate::zmq_client::ZmqClient;
use crate::logging;
use std::time::Instant;

pub struct IcapServer {
    pub listen_addr: String,
    pub zmq_client: ZmqClient,
}

impl IcapServer {
    pub fn new(listen_addr: String, zmq_client: ZmqClient) -> Self {
        IcapServer { listen_addr, zmq_client }
    }

    pub fn run(&self) {
        let listener = TcpListener::bind(&self.listen_addr).expect("Failed to bind ICAP port");
        logging::log_info(&format!("ICAP server listening on {}", self.listen_addr));
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let zmq_client = self.zmq_client.clone();
                    std::thread::spawn(move || {
                        let start = Instant::now();
                        let peer = stream.peer_addr().unwrap_or_else(|_| SocketAddr::from(([0,0,0,0],0)));
                        logging::log_info(&format!("Accepted connection from {}", peer));
                        let mut buf = [0u8; 65536];
                        let n = match stream.read(&mut buf) {
                            Ok(n) if n == 0 => return,
                            Ok(n) => n,
                            Err(e) => {
                                logging::log_error(&format!("Failed to read: {}", e));
                                return;
                            }
                        };
                        let req = &buf[..n];
                        let req_str = String::from_utf8_lossy(req);
                        logging::log_info(&format!("ICAP request ({} bytes):\n{}", n, &req_str));
                        // Parse ICAP method
                        let method = if req_str.starts_with("REQMOD") {
                            "REQMOD"
                        } else if req_str.starts_with("RESPMOD") {
                            "RESPMOD"
                        } else {
                            let _ = stream.write_all(b"ICAP/1.0 500 Server Error\r\n\r\n");
                            return;
                        };
                        // Extract Encapsulated header and HTTP body
                        let encapsulated = req_str.lines().find(|l| l.to_ascii_lowercase().starts_with("encapsulated:"));
                        let http_start = req_str.find("\r\n\r\n");
                        let http_body = if let Some(pos) = http_start {
                            &req[pos+4..]
                        } else {
                            b""
                        };
                        // Forward HTTP body to ZMQ worker
                        let zmq_result = zmq_client.send_and_receive(http_body);
                        let duration = start.elapsed();
                        match zmq_result {
                            Ok(reply) => {
                                // For demo: if reply is unchanged, allow; if changed, modify; if contains BLOCKED, reject
                                let response = if reply == http_body {
                                    b"ICAP/1.0 204 No Content\r\n\r\n".to_vec()
                                } else if reply.windows(7).any(|w| w == b"BLOCKED") {
                                    b"ICAP/1.0 403 Forbidden\r\n\r\nBlocked by policy".to_vec()
                                } else {
                                    // 200 OK with modified body
                                    let mut resp = b"ICAP/1.0 200 OK\r\nEncapsulated: res-body=0\r\n\r\n".to_vec();
                                    resp.extend_from_slice(&reply);
                                    resp
                                };
                                let _ = stream.write_all(&response);
                                logging::log_info(&format!("Responded to {} in {:?}", peer, duration));
                            }
                            Err(e) => {
                                let _ = stream.write_all(b"ICAP/1.0 500 Server Error\r\n\r\n");
                                logging::log_error(&format!("ZMQ error: {}", e));
                            }
                        }
                    });
                }
                Err(e) => {
                    logging::log_error(&format!("Connection failed: {}", e));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zmq_client::ZmqClient;

    #[test]
    fn test_icap_server_new() {
        let zmq_client = ZmqClient::new("tcp://127.0.0.1:5555".to_string());
        let server = IcapServer::new("0.0.0.0:1344".to_string(), zmq_client);
        assert_eq!(server.listen_addr, "0.0.0.0:1344");
    }


} 