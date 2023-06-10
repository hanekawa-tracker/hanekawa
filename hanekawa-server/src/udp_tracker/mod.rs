mod codec;

use codec::UdpTrackerCodec;
use hanekawa_common::Config;

use tokio_util::sync::CancellationToken;

pub async fn start(cfg: &Config, kt: CancellationToken) {
    use futures::StreamExt;
    use tokio::net::UdpSocket;
    use tokio_util::udp::UdpFramed;

    let socket = UdpSocket::bind((cfg.bind_ip, cfg.udp_bind_port))
        .await
        .unwrap();
    let mut socket = UdpFramed::new(socket, UdpTrackerCodec {});

    loop {
        tokio::select! {
            _ = kt.cancelled() => {
                break;
            },
            request = socket.next() => {
                if let Some(request) = request {
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
        }
    }
}
