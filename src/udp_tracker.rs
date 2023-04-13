// BEP 15: UDP Tracker Protocol for BitTorrent

use bytes::{BufMut, BytesMut};
use nom::{IResult, sequence::tuple, bytes::streaming::{tag, take}, number::streaming::{be_i32, be_i64, be_i16}, branch::alt, combinator::map, multi::many1};

const PROTOCOL_ID: u64 = 0x41727101980;

pub struct ConnectRequest {
    transaction_id: i32
}

fn parse_connect_request(input: &[u8]) -> IResult<&[u8], ConnectRequest> {
    let (input, (_, _, tid)) = tuple((
      tag(PROTOCOL_ID.to_be_bytes()),
      tag(0_i32.to_be_bytes()),
      be_i32
    ))(input)?;

    Ok((input, ConnectRequest {
        transaction_id: tid
    }))
}

pub struct ConnectResponse {
    transaction_id: i32,
    connection_id: i64
}

fn encode_connect_response(resp: &ConnectResponse, buf: &mut BytesMut) {
    buf.put_i32(0);
    buf.put_i32(resp.transaction_id);
    buf.put_i64(resp.connection_id);
}

pub enum Event {
    Completed,
    Started,
    Stopped
}

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
    port: i16
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
    map(be_i32, |v| {
        match v {
            0 => None,
            _ => Some(v)
        }
    })(input)
}

fn parse_num_want(input: &[u8]) -> IResult<&[u8], Option<i32>> {
    map(be_i32, |v| {
        match v {
            -1 => None,
            _ => Some(v)
        }
    })(input)
}

fn parse_announce_request(input: &[u8]) -> IResult<&[u8], AnnounceRequest> {
    let (input, (
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
      port
    )) = tuple((
      be_i64,
      tag(0_u32.to_be_bytes()),
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
      be_i16
    ))(input)?;

    Ok((input, AnnounceRequest {
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
        port
    }))
}

pub struct AnnounceResponse {
    transaction_id: i32,
    interval: i32,
    leechers: i32,
    seeders: i32,
    peers: Vec<(i32, i16)>
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

pub struct ScrapeRequest {
    connection_id: i64,
    transaction_id: i32,
    info_hashes: Vec<String>
}

fn parse_scrape_request(input: &[u8]) -> IResult<&[u8], ScrapeRequest> {
    let (input, (
      connection_id,
      _,
      transaction_id,
      info_hashes
    )) = tuple((
      be_i64,
      tag(2_u32.to_be_bytes()),
      be_i32,
      many1(parse_20_bit_string)
    ))(input)?;

    Ok((input, ScrapeRequest {
        connection_id,
        transaction_id,
        info_hashes
    }))
}

pub struct InfoHashScrapeData {
    seeders: i32,
    completed: i32,
    leechers: i32,
}

pub struct ScrapeResponse {
    transaction_id: i32,
    data: Vec<InfoHashScrapeData>
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
    message: String
}

fn encode_error_response(resp: &ErrorResponse, buf: &mut BytesMut) {
    buf.put_i32(3);
    buf.put_i32(resp.transaction_id);
    buf.put_slice(resp.message.as_bytes())
}

pub enum Request {
    Connect(ConnectRequest),
    Announce(AnnounceRequest),
    Scrape(ScrapeRequest)
}

pub fn parse_request(input: &[u8]) -> Result<Request, ()> {
    let result = alt((
      map(parse_connect_request, |r| Request::Connect(r)),
      map(parse_announce_request, |r| Request::Announce(r)),
      map(parse_scrape_request, |r| Request::Scrape(r)),
    ))(input);

    match result {
        Ok((_, req)) => Ok(req),
        _ => Err(())
    }
}

pub enum Response {
    Connect(ConnectResponse),
    Announce(AnnounceResponse),
    Scrape(ScrapeResponse),
    Error(ErrorResponse)
}

pub fn encode_response(response: &Response, buf: &mut BytesMut) {
    use Response::*;
    
    match response {
        Connect(r) => encode_connect_response(r, buf),
        Announce(r) => encode_announce_response(r, buf),
        Scrape(r) => encode_scrape_response(r, buf),
        Error(r) => encode_error_response(r, buf)
    }
}
