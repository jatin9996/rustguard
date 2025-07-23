use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::{info, error};
use crate::icap_client::IcapClient;
use crate::config::Config;

pub struct Proxy {
    pub icap_client: IcapClient,
  

impl Proxy {
    pub fn new(config: &Config) -> Self {
        Proxy {
            icap_client: IcapClient::new(config.icap_server_addr.clone(), 5000), 
    
        }
    }

    pub async fn run(&self) {
        let listen_addr = "0.0.0.0:8080"; // TODO: get from config
        let listener = match TcpListener::bind(listen_addr).await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind to {}: {}", listen_addr, e);
                return;
            }
        };
        info!("Proxy listening on {}", listen_addr);

        loop {
            let icap_client = self.icap_client.clone();
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("Accepted connection from {}", addr);
                    tokio::spawn(Self::handle_client(stream, icap_client));
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    async fn handle_client(mut stream: TcpStream, icap_client: IcapClient) {
        let mut buf = [0u8; 8192];
        let n = match stream.read(&mut buf).await {
            Ok(n) if n == 0 => {
                info!("Client closed connection early");
                return;
            }
            Ok(n) => n,
            Err(e) => {
                error!("Failed to read from client: {}", e);
                return;
            }
        };

        // Parse HTTP request (very basic, for demonstration)
        let request_bytes = &buf[..n];
        let request_str = String::from_utf8_lossy(request_bytes);
        if request_str.starts_with("CONNECT ") {
            // TODO: Handle HTTPS (CONNECT) proxying
            let _ = stream.write_all(b"HTTP/1.1 501 Not Implemented\r\n\r\n").await;
            return;
        }

        info!("Received HTTP request:\n{}", request_str);

        // Forward to ICAP server (REQMOD) and get possibly modified request
        let modified_request = match icap_client.reqmod(request_bytes).await {
            Ok(modified_request) => {
                info!("ICAP REQMOD succeeded, got modified request ({} bytes)", modified_request.len());
                modified_request
            }
            Err(e) => {
                error!("ICAP REQMOD failed: {}. Using original request as fallback.", e);
                request_bytes.to_vec()
            }
        };

        // Parse the host and port from the HTTP request
        let mut lines = request_str.lines();
        let request_line = lines.next().unwrap_or("");
        let mut host = None;
        let mut port = 80; // default HTTP port
        for line in lines {
            if line.to_lowercase().starts_with("host:") {
                let host_header = line[5..].trim();
                if let Some((h, p)) = host_header.split_once(":") {
                    host = Some(h.trim().to_string());
                    if let Ok(parsed_port) = p.trim().parse::<u16>() {
                        port = parsed_port;
                    }
                } else {
                    host = Some(host_header.to_string());
                }
                break;
            }
        }
        let host = match host {
            Some(h) => h,
            None => {
                let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").await;
                error!("No Host header found in request");
                return;
            }
        };

        // Connect to the destination server
        let dest_addr = format!("{}:{}", host, port);
        let mut server_stream = match TcpStream::connect(&dest_addr).await {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to destination {}: {}", dest_addr, e);
                let _ = stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
                return;
            }
        };

        // Send the (possibly modified) request to the destination server
        if let Err(e) = server_stream.write_all(&modified_request).await {
            error!("Failed to send request to destination: {}", e);
            let _ = stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
            return;
        }

        // Read the response from the destination server
        let mut response = Vec::new();
        match server_stream.read_to_end(&mut response).await {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to read response from destination: {}", e);
                let _ = stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
                return;
            }
        }

    
        // Relay the response back to the client
        if let Err(e) = stream.write_all(&response).await {
            error!("Failed to send response to client: {}", e);
        }
    }
}

impl Clone for IcapClient {
    fn clone(&self) -> Self {
        IcapClient::new(self.server_addr.clone(), self.timeout_ms)
    }
} 

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tokio::io::{DuplexStream, duplex};
    use std::sync::Arc;

    #[test]
    fn test_proxy_new() {
        let config = Config {
            icap_server_addr: "127.0.0.1:1344".to_string(),
            listen_addr: "0.0.0.0:8080".to_string(),
            fallback_mode: false,
        };
        let proxy = Proxy::new(&config);
        assert_eq!(proxy.icap_client.server_addr, "127.0.0.1:1344");
    }

    #[async_trait::async_trait]
    pub trait IcapClientTrait: Send + Sync {
        async fn reqmod(&self, data: &[u8]) -> Result<Vec<u8>, String>;
    }

    #[async_trait::async_trait]
    impl IcapClientTrait for IcapClient {
        async fn reqmod(&self, data: &[u8]) -> Result<Vec<u8>, String> {
            self.reqmod(data).await
        }
    }

    struct MockIcapClient;

    #[async_trait::async_trait]
    impl IcapClientTrait for MockIcapClient {
        async fn reqmod(&self, data: &[u8]) -> Result<Vec<u8>, String> {
            // Just echo back the data for testing
            Ok(data.to_vec())
        }
    }

    #[tokio::test]
    async fn test_handle_client() {
        // Create in-memory duplex streams for client <-> proxy
        let (mut client_stream, proxy_stream) = duplex(8192);

        // Write a simple HTTP request to the client side
        let request = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        client_stream.write_all(request).await.unwrap();

        // Use the mock ICAP client
        let icap_client = Arc::new(MockIcapClient);

        // Spawn the proxy handler
        let handle = tokio::spawn(async move {
            handle_client(proxy_stream, icap_client).await;
        });

        // Read the response from the client side
        let mut response = Vec::new();
        client_stream.read_to_end(&mut response).await.unwrap();

        // Assert something about the response
        assert!(response.starts_with(b"HTTP/1.1"));
        handle.await.unwrap();
    }
} 