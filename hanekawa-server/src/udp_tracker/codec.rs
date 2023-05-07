use hanekawa::udp_tracker::proto::{Request, Response};
use hanekawa_udp::{encode_response, parse_request};

use tokio_util::codec::{Decoder, Encoder};

pub struct UdpTrackerCodec;

impl Decoder for UdpTrackerCodec {
    type Item = Request;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        use bytes::Buf;

        if src.remaining() == 0 {
            return Ok(None);
        }

        let req = parse_request(&src);
        src.advance(src.remaining());

        match req {
            Ok(r) => Ok(Some(r)),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }
}

impl Encoder<Response> for UdpTrackerCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Response, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        Ok(encode_response(&item, dst))
    }
}
