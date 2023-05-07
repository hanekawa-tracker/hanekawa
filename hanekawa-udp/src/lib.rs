mod extensions;

use hanekawa::types::Event;
use hanekawa::udp_tracker::proto::*;

use extensions::parse_extensions;

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

const PROTOCOL_ID: u64 = 0x41727101980;

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

fn encode_connect_response(resp: &ConnectResponse, buf: &mut BytesMut) {
    buf.put_i32(0);
    buf.put_i32(resp.transaction_id);
    buf.put_i64(resp.connection_id);
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

fn encode_scrape_response(resp: &ScrapeResponse, buf: &mut BytesMut) {
    buf.put_i32(2);
    buf.put_i32(resp.transaction_id);
    for data in &resp.data {
        buf.put_i32(data.seeders);
        buf.put_i32(data.completed);
        buf.put_i32(data.leechers);
    }
}

fn encode_error_response(resp: &ErrorResponse, buf: &mut BytesMut) {
    buf.put_i32(3);
    buf.put_i32(resp.transaction_id);
    buf.put_slice(resp.message.as_bytes())
}

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

pub fn encode_response(response: &Response, buf: &mut BytesMut) {
    use Response::*;

    match response {
        Connect(r) => encode_connect_response(r, buf),
        Announce(r) => encode_announce_response(r, buf),
        Scrape(r) => encode_scrape_response(r, buf),
        Error(r) => encode_error_response(r, buf),
    }
}
