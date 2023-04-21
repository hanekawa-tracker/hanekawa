// BEP 15: UDP Tracker Protocol for BitTorrent

use bytes::{BufMut, BytesMut};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    combinator::{all_consuming, map},
    multi::many1,
    number::complete::{be_i16, be_i32, be_i64},
    sequence::tuple,
    IResult,
};

use crate::types::Event;

use super::extension_bits::{parse_extensions, Extension};

const PROTOCOL_ID: u64 = 0x41727101980;

#[derive(Debug, Eq, PartialEq)]
pub struct ConnectRequest {
    transaction_id: i32,
}

fn parse_connect_request(input: &[u8]) -> IResult<&[u8], ConnectRequest> {
    let (input, (_, _, tid)) = tuple((
        tag(PROTOCOL_ID.to_be_bytes()),
        tag(0_i32.to_be_bytes()),
        be_i32,
    ))(input)?;

    Ok((
        input,
        ConnectRequest {
            transaction_id: tid,
        },
    ))
}

pub struct ConnectResponse {
    transaction_id: i32,
    connection_id: i64,
}

fn encode_connect_response(resp: &ConnectResponse, buf: &mut BytesMut) {
    buf.put_i32(0);
    buf.put_i32(resp.transaction_id);
    buf.put_i64(resp.connection_id);
}

#[derive(Debug, Eq, PartialEq)]
pub struct AnnounceRequest {
    connection_id: i64,
    transaction_id: i32,
    info_hash: String,
    peer_id: String,
    downloaded: i64,
    left: i64,
    uploaded: i64,
    event: Option<Event>,
    ip_address: Option<i32>,
    key: i32,
    num_want: Option<i32>,
    port: i16,
    extensions: Vec<Extension>,
}

fn parse_20_bit_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, bts) = take(20_usize)(input)?;
    let s = std::str::from_utf8(bts).unwrap();

    Ok((input, s.to_string()))
}

fn parse_event(input: &[u8]) -> IResult<&[u8], Option<Event>> {
    alt((
        map(tag(0_u32.to_be_bytes()), |_| None),
        map(tag(1_u32.to_be_bytes()), |_| Some(Event::Completed)),
        map(tag(2_u32.to_be_bytes()), |_| Some(Event::Started)),
        map(tag(3_u32.to_be_bytes()), |_| Some(Event::Stopped)),
    ))(input)
}

fn parse_ip(input: &[u8]) -> IResult<&[u8], Option<i32>> {
    map(be_i32, |v| match v {
        0 => None,
        _ => Some(v),
    })(input)
}

fn parse_num_want(input: &[u8]) -> IResult<&[u8], Option<i32>> {
    map(be_i32, |v| match v {
        -1 => None,
        _ => Some(v),
    })(input)
}

fn parse_announce_request(input: &[u8]) -> IResult<&[u8], AnnounceRequest> {
    let (
        input,
        (
            connection_id,
            _,
            transaction_id,
            info_hash,
            peer_id,
            downloaded,
            left,
            uploaded,
            event,
            ip_address,
            key,
            num_want,
            port,
            extensions,
        ),
    ) = tuple((
        be_i64,
        tag(1_u32.to_be_bytes()),
        be_i32,
        parse_20_bit_string,
        parse_20_bit_string,
        be_i64,
        be_i64,
        be_i64,
        parse_event,
        parse_ip,
        be_i32,
        parse_num_want,
        be_i16,
        parse_extensions,
    ))(input)?;

    Ok((
        input,
        AnnounceRequest {
            connection_id,
            transaction_id,
            info_hash,
            peer_id,
            downloaded,
            left,
            uploaded,
            event,
            ip_address,
            key,
            num_want,
            port,
            extensions,
        },
    ))
}

pub struct AnnounceResponse {
    transaction_id: i32,
    interval: i32,
    leechers: i32,
    seeders: i32,
    peers: Vec<(i32, i16)>,
}

fn encode_announce_response(resp: &AnnounceResponse, buf: &mut BytesMut) {
    buf.put_i32(1);
    buf.put_i32(resp.transaction_id);
    buf.put_i32(resp.interval);
    buf.put_i32(resp.leechers);
    buf.put_i32(resp.seeders);

    for (ip, port) in &resp.peers {
        buf.put_i32(*ip);
        buf.put_i16(*port);
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ScrapeRequest {
    connection_id: i64,
    transaction_id: i32,
    info_hashes: Vec<String>,
}

fn parse_scrape_request(input: &[u8]) -> IResult<&[u8], ScrapeRequest> {
    let (input, (connection_id, _, transaction_id, info_hashes)) = tuple((
        be_i64,
        tag(2_u32.to_be_bytes()),
        be_i32,
        many1(parse_20_bit_string),
    ))(input)?;

    Ok((
        input,
        ScrapeRequest {
            connection_id,
            transaction_id,
            info_hashes,
        },
    ))
}

pub struct InfoHashScrapeData {
    seeders: i32,
    completed: i32,
    leechers: i32,
}

pub struct ScrapeResponse {
    transaction_id: i32,
    data: Vec<InfoHashScrapeData>,
}

fn encode_scrape_response(resp: &ScrapeResponse, buf: &mut BytesMut) {
    buf.put_i32(2);
    buf.put_i32(resp.transaction_id);
    for data in &resp.data {
        buf.put_i32(data.seeders);
        buf.put_i32(data.completed);
        buf.put_i32(data.leechers);
    }
}

pub struct ErrorResponse {
    transaction_id: i32,
    message: String,
}

fn encode_error_response(resp: &ErrorResponse, buf: &mut BytesMut) {
    buf.put_i32(3);
    buf.put_i32(resp.transaction_id);
    buf.put_slice(resp.message.as_bytes())
}

#[derive(Debug)]
pub enum Request {
    Connect(ConnectRequest),
    Announce(AnnounceRequest),
    Scrape(ScrapeRequest),
}

#[derive(Debug)]
pub enum Error {
    Other(()),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Other error")
    }
}

impl std::error::Error for Error {}

pub fn parse_request(input: &[u8]) -> Result<Request, Error> {
    let result = all_consuming(alt((
        map(parse_connect_request, |r| Request::Connect(r)),
        map(parse_announce_request, |r| Request::Announce(r)),
        map(parse_scrape_request, |r| Request::Scrape(r)),
    )))(input);

    match result {
        Ok((_, req)) => Ok(req),
        _ => Err(Error::Other(())),
    }
}

pub enum Response {
    Connect(ConnectResponse),
    Announce(AnnounceResponse),
    Scrape(ScrapeResponse),
    Error(ErrorResponse),
}

pub fn encode_response(response: &Response, buf: &mut BytesMut) {
    use Response::*;

    match response {
        Connect(r) => encode_connect_response(r, buf),
        Announce(r) => encode_announce_response(r, buf),
        Scrape(r) => encode_scrape_response(r, buf),
        Error(r) => encode_error_response(r, buf),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_connection_request() {
        let mut buf = BytesMut::new();

        buf.put_u64(PROTOCOL_ID);
        buf.put_i32(0);
        buf.put_i32(42);

        assert_eq!(
            Ok((&[] as &[u8], ConnectRequest { transaction_id: 42 })),
            parse_connect_request(&buf)
        )
    }

    #[test]
    fn parses_announce_request() {
        let mut buf = BytesMut::new();

        let peer_id = "12345678901234567890";
        let info_hash = "09876543210987654321";

        buf.put_i64(42);
        buf.put_i32(1);
        buf.put_i32(32);
        buf.put_slice(info_hash.as_bytes());
        buf.put_slice(peer_id.as_bytes());
        buf.put_i64(3);
        buf.put_i64(4);
        buf.put_i64(5);
        buf.put_i32(3);
        buf.put_i32(0);
        buf.put_i32(17);
        buf.put_i32(-1);
        buf.put_i16(3001);

        assert_eq!(
            Ok((
                &[] as &[u8],
                AnnounceRequest {
                    connection_id: 42,
                    transaction_id: 32,
                    info_hash: info_hash.to_string(),
                    peer_id: peer_id.to_string(),
                    downloaded: 3,
                    left: 4,
                    uploaded: 5,
                    event: Some(Event::Stopped),
                    ip_address: None,
                    key: 17,
                    num_want: None,
                    port: 3001,
                    extensions: Vec::new()
                }
            )),
            parse_announce_request(&buf)
        )
    }

    #[test]
    fn parses_scrape_request() {
        let mut buf = BytesMut::new();

        let info_hash = "01234567890123456789".to_string();
        let num_hashes = 6;

        let mut hashes = Vec::new();
        for _ in 0..num_hashes {
            hashes.push(info_hash.clone());
        }

        buf.put_i64(42);
        buf.put_i32(2);
        buf.put_i32(32);
        for _ in 0..num_hashes {
            buf.put_slice(info_hash.as_bytes())
        }

        assert_eq!(
            Ok((
                &[] as &[u8],
                ScrapeRequest {
                    connection_id: 42,
                    transaction_id: 32,
                    info_hashes: hashes
                }
            )),
            parse_scrape_request(&buf)
        )
    }
}
