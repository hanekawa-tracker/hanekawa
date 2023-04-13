pub mod bencode;
mod server;
mod types;
mod udp_tracker;

pub async fn start() {
    let sh = tokio::spawn(async move {
        server::start().await;
    });

    let uh = tokio::spawn(async move {
        udp_tracker::start().await;
    });

    tokio::join!(sh, uh);
}
