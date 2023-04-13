pub mod bencode;
mod server;
mod udp_tracker;

pub async fn start() {
    server::start().await;
}
