mod ser;

pub use ser::to_string;

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    String(String),
    Int(i64),
    List(Vec<Value>),
    Dict(BTreeMap<String, Value>),
}

fn is_numeric(c: char) -> bool {
    c.is_digit(10)
}

fn parse_numeric(input: &str) -> IResult<&str, u32> {
    let (input, len) = take_while1(is_numeric)(input)?;
    let len: u32 = len.parse().unwrap();
    Ok((input, len))
}

// parses <len>:<str>
fn parse_string(input: &str) -> IResult<&str, String> {
    use nom::error::{Error, ErrorKind};

    let (input, len) = parse_numeric(input)?;
    let (input, _) = tag(":")(input)?;

    if input.len() >= len as usize {
        let (s, rest) = input.split_at(len as usize);
        return Ok((rest, s.to_string()));
    }

    Err(nom::Err::Failure(Error::new(input, ErrorKind::Eof)))
}

fn encode_string(s: &str, buf: &mut BytesMut) {
    buf.put_slice(&s.len().to_string().as_bytes());
    buf.put_slice(":".as_bytes());
    buf.put_slice(s.as_bytes());
}

fn parse_string_value(input: &str) -> IResult<&str, Value> {
    map(parse_string, |s| Value::String(s))(input)
}

fn parse_integer_numeric_part(input: &str) -> IResult<&str, i64> {
    fn is_nonzero_numeric(c: char) -> bool {
        is_numeric(c) && c != '0'
    }

    let (input, matched) = alt((
        recognize(tag("0")),
        recognize(tuple((
            opt(tag("-")),
            take_while_m_n(1, 1, is_nonzero_numeric),
            take_while(is_numeric),
        ))),
    ))(input)?;

    let matched: i64 = matched.parse().unwrap();

    Ok((input, matched))
}

// parses i<num>e
fn parse_integer(input: &str) -> IResult<&str, Value> {
    let result = delimited(
        tag("i"),
        map(parse_integer_numeric_part, |i| Value::Int(i)),
        tag("e"),
    )(input)?;

    Ok(result)
}

fn encode_integer(i: i64, buf: &mut BytesMut) {
    buf.put_slice(format!("i{}e", i).as_bytes());
}

// parses l<value*>e
fn parse_list(input: &str) -> IResult<&str, Value> {
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
fn parse_dict(input: &str) -> IResult<&str, Value> {
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
        encode_string(k, buf);
        encode_value(v, buf);
    }
    buf.put_slice("e".as_bytes())
}

fn parse_value(input: &str) -> IResult<&str, Value> {
    alt((parse_string_value, parse_integer, parse_list, parse_dict))(input)
}

fn encode_value(value: &Value, buf: &mut BytesMut) {
    match value {
        Value::String(s) => encode_string(s, buf),
        Value::Int(i) => encode_integer(*i, buf),
        Value::List(vs) => encode_list(vs, buf),
        Value::Dict(vs) => encode_dict(vs, buf),
    }
}

pub fn encode(value: &Value) -> String {
    let mut buf = BytesMut::new();
    encode_value(value, &mut buf);

    std::str::from_utf8(&buf).unwrap().to_string()
}

pub fn parse(input: &str) -> Result<Value, ()> {
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
        let enc = "4:spam";
        assert_eq!(Value::String("spam".to_string()), parse(&enc).unwrap())
    }

    #[test]
    fn parses_valid_ints() {
        assert_eq!(Value::Int(3), parse("i3e").unwrap());
        assert_eq!(Value::Int(0), parse("i0e").unwrap())
    }

    #[test]
    fn rejects_invalid_ints() {
        assert!(parse("i03e").is_err(), "leading zeros are invalid");
        assert!(parse("i-0e").is_err(), "negative zero is invalid");
    }

    #[test]
    fn parses_lists() {
        let enc = "l4:spam4:eggse";
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
            parse("d3:cow3:moo4:spam4:eggse").unwrap()
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
            parse("d4:spaml1:a1:bee").unwrap()
        );
    }
}
