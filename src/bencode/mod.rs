mod ser;

pub use ser::to_bytes;

use std::collections::BTreeMap;

use bytes::{BufMut, BytesMut};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1, take_while_m_n},
    combinator::{all_consuming, map, opt, recognize},
    multi::many0,
    sequence::{delimited, terminated, tuple},
    IResult,
};

#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, Eq)]
pub enum Value {
    Bytes(Vec<u8>),
    String(String),
    Int(i64),
    List(Vec<Value>),
    Dict(BTreeMap<String, Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bytes(b), Self::Bytes(b2)) if b == b2 => true,
            (Self::Bytes(b), Self::String(s)) if b == s.as_bytes() => true,
            (Self::String(s), Self::Bytes(b)) if b == s.as_bytes() => true,
            (Self::String(s), Self::String(s2)) if s == s2 => true,
            (Self::Int(i), Self::Int(i2)) if i == i2 => true,
            (Self::List(l), Self::List(l2)) if l == l2 => true,
            (Self::Dict(d), Self::Dict(d2)) if d == d2 => true,
            _ => false,
        }
    }
}

fn is_numeric(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

fn parse_numeric(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, len) = take_while1(is_numeric)(input)?;
    let len = std::str::from_utf8(len).unwrap();
    let len: u32 = len.parse().unwrap();
    Ok((input, len))
}

// parses <len>:<str>
fn parse_string_or_bytes_value(input: &[u8]) -> IResult<&[u8], Value> {
    use nom::error::{Error, ErrorKind};

    let (input, len) = parse_numeric(input)?;
    let (input, _) = tag(":")(input)?;

    if input.len() >= len as usize {
        let (s, rest) = input.split_at(len as usize);
        return match std::str::from_utf8(s) {
            Ok(s) => Ok((rest, Value::String(s.to_string()))),
            _ => Ok((rest, Value::Bytes(s.to_vec()))),
        };
    }

    Err(nom::Err::Failure(Error::new(input, ErrorKind::Eof)))
}

fn parse_string(input: &[u8]) -> IResult<&[u8], String> {
    use nom::error::{Error, ErrorKind};

    let (input, val) = parse_string_or_bytes_value(input)?;

    match val {
        Value::String(s) => Ok((input, s)),
        _ => Err(nom::Err::Failure(Error::new(input, ErrorKind::Verify))),
    }
}

fn encode_string(bytes: &[u8], buf: &mut BytesMut) {
    let mut ws = itoa::Buffer::new();
    buf.put_slice(ws.format(bytes.len()).as_bytes());

    buf.put_slice(":".as_bytes());
    buf.put_slice(bytes);
}

fn parse_integer_numeric_part(input: &[u8]) -> IResult<&[u8], i64> {
    fn is_nonzero_numeric(c: u8) -> bool {
        is_numeric(c) && c != b'0'
    }

    let (input, matched) = alt((
        recognize(tag("0")),
        recognize(tuple((
            opt(tag("-")),
            take_while_m_n(1, 1, is_nonzero_numeric),
            take_while(is_numeric),
        ))),
    ))(input)?;

    let matched = std::str::from_utf8(matched).unwrap();
    let matched: i64 = matched.parse().unwrap();

    Ok((input, matched))
}

// parses i<num>e
fn parse_integer(input: &[u8]) -> IResult<&[u8], Value> {
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
fn parse_list(input: &[u8]) -> IResult<&[u8], Value> {
    delimited(
        tag("l"),
        map(many0(parse_value), |vs| Value::List(vs)),
        tag("e"),
    )(input)
}

fn encode_list(vs: &Vec<Value>, buf: &mut BytesMut) {
    buf.put_slice("l".as_bytes());
    for v in vs {
        encode_value(v, buf);
    }
    buf.put_slice("e".as_bytes())
}

// d<(<str><value>)*>e
fn parse_dict(input: &[u8]) -> IResult<&[u8], Value> {
    delimited(
        tag("d"),
        map(many0(tuple((parse_string, parse_value))), |ps| {
            Value::Dict(ps.into_iter().collect())
        }),
        tag("e"),
    )(input)
}

fn encode_dict(vs: &BTreeMap<String, Value>, buf: &mut BytesMut) {
    buf.put_slice("d".as_bytes());
    for (k, v) in vs {
        encode_string(k.as_bytes(), buf);
        encode_value(v, buf);
    }
    buf.put_slice("e".as_bytes())
}

fn parse_value(input: &[u8]) -> IResult<&[u8], Value> {
    alt((
        parse_string_or_bytes_value,
        parse_integer,
        parse_list,
        parse_dict,
    ))(input)
}

fn encode_value(value: &Value, buf: &mut BytesMut) {
    match value {
        Value::Bytes(b) => encode_string(&b, buf),
        Value::String(s) => encode_string(s.as_bytes(), buf),
        Value::Int(i) => encode_integer(*i, buf),
        Value::List(vs) => encode_list(vs, buf),
        Value::Dict(vs) => encode_dict(vs, buf),
    }
}

pub fn encode(value: &Value) -> Vec<u8> {
    let mut buf = BytesMut::new();
    encode_value(value, &mut buf);

    buf.to_vec()
}

pub fn parse(input: &[u8]) -> Result<Value, ()> {
    let result = all_consuming(terminated(parse_value, opt(tag("\n"))))(input);
    match result {
        Ok((_, v)) => Ok(v),
        _ => Err(()),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_string() {
        let enc = "4:spam".as_bytes();
        assert_eq!(Value::String("spam".to_string()), parse(&enc).unwrap())
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
                Value::String("spam".to_string()),
                Value::String("eggs".to_string())
            ]),
            parse(enc).unwrap()
        )
    }

    #[test]
    fn parses_dicts() {
        assert_eq!(
            Value::Dict(
                vec![
                    ("cow".to_string(), Value::String("moo".to_string())),
                    ("spam".to_string(), Value::String("eggs".to_string()))
                ]
                .into_iter()
                .collect()
            ),
            parse("d3:cow3:moo4:spam4:eggse".as_bytes()).unwrap()
        );

        assert_eq!(
            Value::Dict(
                vec![(
                    "spam".to_string(),
                    Value::List(vec![
                        Value::String("a".to_string()),
                        Value::String("b".to_string())
                    ])
                )]
                .into_iter()
                .collect()
            ),
            parse("d4:spaml1:a1:bee".as_bytes()).unwrap()
        );
    }
}
