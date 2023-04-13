pub mod bencode;
mod server;

pub async fn start() {
    server::start().await;
}
