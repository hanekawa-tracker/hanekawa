mod ser;

pub use ser::{to_bytes, value_to_bytes};

use std::collections::BTreeMap;

use bytes::{BufMut, BytesMut};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{all_consuming, map, opt, recognize, verify},
    multi::many0,
    sequence::{delimited, terminated, tuple},
    IResult,
};

#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value<B: Ord> {
    Bytes(B),
    Int(i64),
    List(Vec<Self>),
    Dict(BTreeMap<B, Self>),
}

fn parse_numeric(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, len) = digit1(input)?;
    let len = unsafe {
        // ASCII digits are always valid UTF-8
        std::str::from_utf8_unchecked(len)
    };
    let len: u32 = len.parse().unwrap();
    Ok((input, len))
}

fn parse_bytes(input: &[u8]) -> IResult<&[u8], &[u8]> {
    use nom::error::{Error, ErrorKind};

    let (input, len) = parse_numeric(input)?;
    let (input, _) = tag(":")(input)?;

    if input.len() >= len as usize {
        let (s, rest) = input.split_at(len as usize);
        return Ok((rest, s));
    }

    Err(nom::Err::Failure(Error::new(input, ErrorKind::Eof)))
}

// parses <len>:<str>
fn parse_string_or_bytes_value(input: &[u8]) -> IResult<&[u8], Value<&[u8]>> {
    map(parse_bytes, |bs| Value::Bytes(bs))(input)
}

fn encode_string<B: AsRef<[u8]> + Ord>(bytes: B, buf: &mut BytesMut) {
    let bytes = bytes.as_ref();
    let mut ws = itoa::Buffer::new();
    buf.put_slice(ws.format(bytes.len()).as_bytes());

    buf.put_u8(b':');
    buf.put_slice(&bytes);
}

fn parse_integer_numeric_part(input: &[u8]) -> IResult<&[u8], i64> {
    let (input, matched) = alt((
        recognize(tag("0")),
        recognize(tuple((
            opt(tag("-")),
            verify(digit1, |ds: &[u8]| ds[0] != '0' as u8),
        ))),
    ))(input)?;

    let matched = unsafe {
        // [+]ASCII digits are always valid UTF-8.
        std::str::from_utf8_unchecked(matched)
    };
    let matched: i64 = matched.parse().unwrap();

    Ok((input, matched))
}

// parses i<num>e
fn parse_integer(input: &[u8]) -> IResult<&[u8], Value<&[u8]>> {
    let result = delimited(
        tag("i"),
        map(parse_integer_numeric_part, |i| Value::Int(i)),
        tag("e"),
    )(input)?;

    Ok(result)
}

fn encode_integer(i: i64, buf: &mut BytesMut) {
    buf.put_slice("i".as_bytes());
    let mut ws = itoa::Buffer::new();
    buf.put_slice(ws.format(i).as_bytes());
    buf.put_slice("e".as_bytes());
}

// parses l<value*>e
fn parse_list(input: &[u8]) -> IResult<&[u8], Value<&[u8]>> {
    delimited(
        tag("l"),
        map(many0(parse_value), |vs| Value::List(vs)),
        tag("e"),
    )(input)
}

fn encode_list<B: AsRef<[u8]> + Ord>(vs: &Vec<Value<B>>, buf: &mut BytesMut) {
    buf.put_slice("l".as_bytes());
    for v in vs {
        encode_value(v, buf);
    }
    buf.put_slice("e".as_bytes())
}

// d<(<str><value>)*>e
fn parse_dict(input: &[u8]) -> IResult<&[u8], Value<&[u8]>> {
    delimited(
        tag("d"),
        map(many0(tuple((parse_bytes, parse_value))), |ps| {
            Value::Dict(ps.into_iter().collect())
        }),
        tag("e"),
    )(input)
}

fn encode_dict<B: AsRef<[u8]> + Ord>(vs: &BTreeMap<B, Value<B>>, buf: &mut BytesMut) {
    buf.put_slice("d".as_bytes());
    for (k, v) in vs {
        encode_string(k, buf);
        encode_value(v, buf);
    }
    buf.put_slice("e".as_bytes())
}

fn parse_value(input: &[u8]) -> IResult<&[u8], Value<&[u8]>> {
    alt((
        parse_string_or_bytes_value,
        parse_integer,
        parse_list,
        parse_dict,
    ))(input)
}

fn encode_value<B: AsRef<[u8]> + Ord>(value: &Value<B>, buf: &mut BytesMut) {
    match value {
        Value::Bytes(b) => encode_string(&b, buf),
        // Value::String(s) => encode_string(s.as_bytes(), buf),
        Value::Int(i) => encode_integer(*i, buf),
        Value::List(vs) => encode_list(vs, buf),
        Value::Dict(vs) => encode_dict(vs, buf),
    }
}

pub fn encode<B: AsRef<[u8]> + Ord>(value: &Value<B>) -> Vec<u8> {
    let mut buf = BytesMut::new();
    encode_value(value, &mut buf);

    buf.to_vec()
}

pub fn parse(input: &[u8]) -> Result<Value<&[u8]>, ()> {
    let result = all_consuming(terminated(parse_value, opt(tag("\n"))))(input);
    match result {
        Ok((_, v)) => Ok(v),
        e => Err(()),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_string() {
        let enc = "4:spam".as_bytes();
        assert_eq!(Value::Bytes("spam".as_bytes()), parse(&enc).unwrap())
    }

    #[test]
    fn parses_valid_ints() {
        assert_eq!(Value::Int(3), parse("i3e".as_bytes()).unwrap());
        assert_eq!(Value::Int(0), parse("i0e".as_bytes()).unwrap())
    }

    #[test]
    fn rejects_invalid_ints() {
        assert!(
            parse("i03e".as_bytes()).is_err(),
            "leading zeros are invalid"
        );
        assert!(
            parse("i-0e".as_bytes()).is_err(),
            "negative zero is invalid"
        );
    }

    #[test]
    fn parses_lists() {
        let enc = "l4:spam4:eggse".as_bytes();
        assert_eq!(
            Value::List(vec![
                Value::Bytes("spam".as_bytes()),
                Value::Bytes("eggs".as_bytes())
            ]),
            parse(enc).unwrap()
        )
    }

    #[test]
    fn parses_dicts() {
        assert_eq!(
            Value::Dict(
                vec![
                    ("cow".as_bytes(), Value::Bytes("moo".as_bytes())),
                    ("spam".as_bytes(), Value::Bytes("eggs".as_bytes()))
                ]
                .into_iter()
                .collect()
            ),
            parse("d3:cow3:moo4:spam4:eggse".as_bytes()).unwrap()
        );

        assert_eq!(
            Value::Dict(
                vec![(
                    "spam".as_bytes(),
                    Value::List(vec![
                        Value::Bytes("a".as_bytes()),
                        Value::Bytes("b".as_bytes())
                    ])
                )]
                .into_iter()
                .collect()
            ),
            parse("d4:spaml1:a1:bee".as_bytes()).unwrap()
        );
    }
}
