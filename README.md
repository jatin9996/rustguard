# rustguard-icap

A Rust-based ICAP (Internet Content Adaptation Protocol) server and proxy framework with ZeroMQ integration for content inspection and modification.

## Overview

**rustguard-icap** provides a modular framework for intercepting, inspecting, and modifying HTTP traffic using the ICAP protocol. It is designed for use cases such as web content filtering, malware/phishing detection, and policy enforcement. The project leverages ZeroMQ for scalable, decoupled worker communication, allowing for flexible content analysis and modification pipelines.

## Features
- ICAP server implementation in Rust
- HTTP proxy with ICAP REQMOD/RESPMOD support
- ZeroMQ-based worker integration for content inspection
- Simple content inspection logic (block, modify, or allow)
- Configurable addresses and fallback modes
- Logging and utility helpers

## Project Structure

- `main.rs` — Entry point; loads configuration, starts ICAP server
- `icap_server.rs` — ICAP server implementation, handles ICAP requests and communicates with workers
- `icap_client.rs` — ICAP client for proxying HTTP requests through ICAP
- `proxy.rs` — HTTP proxy that forwards requests to ICAP server for inspection
- `zmq_client.rs` — ZeroMQ client for sending content to workers
- `worker.rs` — ZeroMQ worker for content inspection (block, modify, or allow)
- `config.rs` — Configuration loading and management
- `logging.rs` — Logging helpers
- `utils.rs` — Utility functions (IP/port validation, random ID generation, etc.)

## Getting Started

### Prerequisites
- Rust (edition 2024 or later)
- ZeroMQ library (for worker and client communication)

### Installation
1. Clone the repository:
   ```sh
   git clone <repo-url>
   cd rustguard-icap
   ```
2. Build the project:
   ```sh
   cargo build --release
   ```

### Running the ICAP Server

1. Start a ZeroMQ worker (in a separate terminal):
   ```sh
   cargo run --bin worker
   ```
   *(Or integrate the worker logic as needed)*

2. Start the ICAP server:
   ```sh
   cargo run --release
   ```

The server listens on the address specified in the configuration (default: `0.0.0.0:8080` for HTTP proxy, `127.0.0.1:1344` for ICAP server).

## Configuration

Configuration is loaded from defaults, but can be extended to support files or environment variables. See `config.rs` for details.

- `icap_server_addr`: Address of the ICAP server (default: `127.0.0.1:1344`)
- `listen_addr`: Address to listen for incoming HTTP/ICAP requests (default: `0.0.0.0:8080`)
- `fallback_mode`: Whether to allow requests if inspection fails (default: `false`)

## Example Content Inspection Logic

The worker inspects content and:
- Blocks if it contains "malware" or "phishing"
- Modifies if it contains "editme"
- Allows otherwise

## Dependencies
- [zmq](https://crates.io/crates/zmq)
- [log](https://crates.io/crates/log)
- [env_logger](https://crates.io/crates/env_logger)

## License

MIT License. See [LICENSE](LICENSE) for details. 