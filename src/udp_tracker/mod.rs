// BEP 15 and BEP 41

mod bits;
mod extension_bits;

use tokio_util::codec::{Decoder, Encoder};

struct UdpTrackerCodec;

impl Decoder for UdpTrackerCodec {
    type Item = bits::Request;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        use bytes::Buf;

        if src.remaining() == 0 {
            return Ok(None);
        }

        let req = bits::parse_request(&src);
        src.advance(src.remaining());

        match req {
            Ok(r) => Ok(Some(r)),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }
}

impl Encoder<bits::Response> for UdpTrackerCodec {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: bits::Response,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        Ok(bits::encode_response(&item, dst))
    }
}

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
