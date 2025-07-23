
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[async_trait::async_trait]
pub trait AsyncStream: Send + Sync {
    async fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error>;
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error>;
}

#[async_trait::async_trait]
impl AsyncStream for TcpStream {
    async fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        AsyncWriteExt::write_all(self, buf).await
    }
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        AsyncReadExt::read(self, buf).await
    }
}

pub struct IcapClient {
  
    pub server_addr: String,

    pub timeout_ms: u64,
}

impl IcapClient {
    pub fn new(server_addr: String, timeout_ms: u64) -> Self {
        IcapClient {
            server_addr,
            timeout_ms,
        }
    }

    pub async fn reqmod(&self, http_request: &[u8]) -> Result<Vec<u8>, String> {
        // Build ICAP REQMOD request
        let icap_req = format!(
            "REQMOD icap://{}/reqmod ICAP/1.0\r\nHost: {}\r\nAllow: 204\r\nEncapsulated: req-body=0\r\n\r\n",
            self.server_addr, self.server_addr
        );
        let mut req_bytes = icap_req.as_bytes().to_vec();
        req_bytes.extend_from_slice(http_request);

        // Connect to ICAP server
        let mut stream = TcpStream::connect(&self.server_addr)
            .await
            .map_err(|e| format!("Failed to connect to ICAP server: {}", e))?;
        stream.write_all(&req_bytes)
            .await
            .map_err(|e| format!("Failed to send to ICAP server: {}", e))?;

        // Read response
        let mut buf = vec![0u8; 65536];
        let n = stream.read(&mut buf)
            .await
            .map_err(|e| format!("Failed to read from ICAP server: {}", e))?;
        let response = &buf[..n];
        let response_str = String::from_utf8_lossy(response);

        // Find the start of the encapsulated HTTP message
        if let Some(pos) = response_str.find("\r\n\r\n") {
            let http_start = pos + 4;
            if http_start < response.len() {
                return Ok(response[http_start..].to_vec());
            }
        }
        Err("ICAP server response did not contain encapsulated HTTP message".to_string())
    }

    pub async fn respmod(&self, http_response: &[u8]) -> Result<Vec<u8>, String> {
        // Build ICAP RESPMOD request
        let icap_req = format!(
            "RESPMOD icap://{}/respmod ICAP/1.0\r\nHost: {}\r\nAllow: 204\r\nEncapsulated: res-body=0\r\n\r\n",
            self.server_addr, self.server_addr
        );
        let mut req_bytes = icap_req.as_bytes().to_vec();
        req_bytes.extend_from_slice(http_response);

        // Connect to ICAP server
        let mut stream = TcpStream::connect(&self.server_addr)
            .await
            .map_err(|e| format!("Failed to connect to ICAP server: {}", e))?;
        stream.write_all(&req_bytes)
            .await
            .map_err(|e| format!("Failed to send to ICAP server: {}", e))?;

        // Read response
        let mut buf = vec![0u8; 65536];
        let n = stream.read(&mut buf)
            .await
            .map_err(|e| format!("Failed to read from ICAP server: {}", e))?;
        let response = &buf[..n];
        let response_str = String::from_utf8_lossy(response);

        // Find the start of the encapsulated HTTP message
        if let Some(pos) = response_str.find("\r\n\r\n") {
            let http_start = pos + 4;
            if http_start < response.len() {
                return Ok(response[http_start..].to_vec());
            }
        }
        Err("ICAP server response did not contain encapsulated HTTP message".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};
    use async_trait::async_trait;

    mock! {
        pub Stream {}
        #[async_trait]
        impl AsyncStream for Stream {
            async fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error>;
            async fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error>;
        }
    }

    #[tokio::test]
    async fn test_reqmod_success() {
        let mut mock_stream = MockStream::new();
        mock_stream.expect_write_all()
            .returning(|_| Ok(()));
        mock_stream.expect_read()
            .returning(|buf| {
                let response = b"ICAP/1.0 200 OK\r\n\r\nHTTP/1.1 200 OK\r\n\r\nbody";
                buf[..response.len()].copy_from_slice(response);
                Ok(response.len())
            });

    }

  
} 