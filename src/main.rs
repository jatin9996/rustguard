mod config;
mod icap_server;
mod zmq_client;
mod worker;
mod logging;

use icap_server::IcapServer;
use zmq_client::ZmqClient;
use worker::Worker;
use config::Config;

fn main() {
    let config = Config::load();
    let zmq_client = ZmqClient::new("tcp://127.0.0.1:5555".to_string());
    let icap_server = IcapServer::new(config.listen_addr, zmq_client);
    
    icap_server.run();
   
