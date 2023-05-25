mod codec;

use codec::UdpTrackerCodec;
use hanekawa_common::Config;

pub async fn start(cfg: &Config) {
    use futures::StreamExt;
    use tokio::net::UdpSocket;
    use tokio_util::udp::UdpFramed;

    let socket = UdpSocket::bind((cfg.bind_ip, cfg.udp_bind_port))
        .await
        .unwrap();
    let mut socket = UdpFramed::new(socket, UdpTrackerCodec {});

    while let Some(request) = socket.next().await {
        match request {
            Ok((request, ip)) => {
                eprintln!("received request {:?} from ip {:?}", request, ip);
            }
            Err(e) => {
                eprintln!("malformed message, {:?}", e);
            }
        }
    }
}
