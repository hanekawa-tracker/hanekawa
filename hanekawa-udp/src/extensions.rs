// BEP 41: UDP Tracker Protocol Extensions

use hanekawa::udp_tracker::proto::Extension;

use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    combinator::{map, opt},
    multi::many0,
    number::complete::be_u8,
    sequence::terminated,
    IResult,
};

fn parse_three_part_option(input: &[u8]) -> IResult<&[u8], (u8, String)> {
    let (input, id) = be_u8(input)?;
    let (input, len) = be_u8(input)?;
    let (input, bs) = take(len)(input)?;
    let s = std::str::from_utf8(bs).unwrap().to_string();

    Ok((input, (id, s)))
}

pub(super) fn parse_extensions(input: &[u8]) -> IResult<&[u8], Vec<Extension>> {
    terminated(
        many0(alt((
            map(tag("\x01"), |_| Extension::Nop),
            map(parse_three_part_option, |(tag, data)| match tag {
                2 => Extension::UrlData(data),
                _ => Extension::Unknown(tag, data),
            }),
        ))),
        opt(tag("\0")),
    )(input)
}

#[cfg(test)]
mod test {
    use bytes::BufMut;
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn parses_implicity_empty_extensions() {
        assert_eq!(Ok((&[] as &[u8], vec![])), parse_extensions(&[]))
    }

    #[test]
    fn parses_explicitly_empty_extensions() {
        let mut buf = BytesMut::new();

        buf.put_u8(0);

        assert_eq!(Ok((&[] as &[u8], vec![])), parse_extensions(&buf))
    }

    #[test]
    fn parses_stops_parsing_empty_extensions() {
        let mut buf = BytesMut::new();

        buf.put_u8(1);
        buf.put_u8(0);
        buf.put_u8(1);

        assert_eq!(
            Ok((&[b'\x01'] as &[u8], vec![Extension::Nop])),
            parse_extensions(&buf)
        )
    }

    #[test]
    fn parses_nop_extension() {
        let mut buf = BytesMut::new();

        buf.put_u8(1);

        assert_eq!(
            Ok((&[] as &[u8], vec![Extension::Nop])),
            parse_extensions(&buf)
        )
    }

    #[test]
    fn parses_urldata() {
        let mut buf = BytesMut::new();

        let opts = "/announce?peer_id=1".to_string();

        buf.put_u8(2);
        buf.put_u8(opts.len() as u8);
        buf.put_slice(opts.as_bytes());

        assert_eq!(
            Ok((&[] as &[u8], vec![Extension::UrlData(opts)])),
            parse_extensions(&buf)
        )
    }

    #[test]
    fn parses_unknown_data() {
        let mut buf = BytesMut::new();

        let opts = "mystery".to_string();

        buf.put_u8(127);
        buf.put_u8(opts.len() as u8);
        buf.put_slice(opts.as_bytes());

        assert_eq!(
            Ok((&[] as &[u8], vec![Extension::Unknown(127, opts)])),
            parse_extensions(&buf)
        )
    }
}
