mod codec;

use codec::UdpTrackerCodec;

pub async fn start() {
    use futures::StreamExt;
    use tokio::net::UdpSocket;
    use tokio_util::udp::UdpFramed;

    let socket = UdpSocket::bind("0.0.0.0:8002").await.unwrap();
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
