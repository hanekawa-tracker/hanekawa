mod ser;

pub use ser::to_bytes;

use std::collections::BTreeMap;

use bytes::{BufMut, BytesMut};

use nom::{
    branch::alt,
    character::complete::{char, digit1},
    combinator::{all_consuming, map, opt, recognize, verify},
    multi::{fold_many0, many0},
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

#[inline]
fn parse_numeric(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, num) = digit1(input)?;
    let num = lexical::parse(&num).unwrap();
    Ok((input, num))
}

#[inline]
fn parse_bytes(input: &[u8]) -> IResult<&[u8], &[u8]> {
    use nom::error::{Error, ErrorKind};

    let (input, len) = parse_numeric(input)?;
    let (input, _) = char(':')(input)?;

    if input.len() >= len as usize {
        let (s, rest) = input.split_at(len as usize);
        return Ok((rest, s));
    }

    Err(nom::Err::Failure(Error::new(input, ErrorKind::Eof)))
}

// parses <len>:<str>
#[inline]
fn parse_bytes_value(input: &[u8]) -> IResult<&[u8], Value<&[u8]>> {
    map(parse_bytes, |bs| Value::Bytes(bs))(input)
}

fn encode_string<B: AsRef<[u8]> + Ord>(bytes: B, buf: &mut BytesMut) {
    use lexical::{FormattedSize, ToLexical};
    let mut digits = [0; usize::FORMATTED_SIZE_DECIMAL];

    let bytes = bytes.as_ref();

    buf.put_slice(bytes.len().to_lexical(&mut digits));
    buf.put_u8(b':');
    buf.put_slice(&bytes);
}

#[inline]
fn parse_integer_numeric_part(input: &[u8]) -> IResult<&[u8], i64> {
    let (input, matched) = alt((
        recognize(char('0')),
        recognize(tuple((
            opt(char('-')),
            verify(digit1, |ds: &[u8]| ds[0] != '0' as u8),
        ))),
    ))(input)?;

    let matched: i64 = lexical::parse(matched).unwrap();

    Ok((input, matched))
}

// parses i<num>e
#[inline]
fn parse_integer(input: &[u8]) -> IResult<&[u8], Value<&[u8]>> {
    let result = delimited(
        char('i'),
        map(parse_integer_numeric_part, |i| Value::Int(i)),
        char('e'),
    )(input)?;

    Ok(result)
}

fn encode_integer(i: i64, buf: &mut BytesMut) {
    use lexical::{FormattedSize, ToLexical};
    let mut digits = [0; usize::FORMATTED_SIZE_DECIMAL];

    buf.put_u8(b'i');
    buf.put_slice(i.to_lexical(&mut digits));
    buf.put_u8(b'e');
}

// parses l<value*>e
#[inline]
fn parse_list(input: &[u8]) -> IResult<&[u8], Value<&[u8]>> {
    delimited(
        char('l'),
        map(many0(parse_value), |vs| Value::List(vs)),
        char('e'),
    )(input)
}

#[inline]
fn encode_list_begin(buf: &mut BytesMut) {
    buf.put_u8(b'l');
}

#[inline]
fn encode_list_end(buf: &mut BytesMut) {
    buf.put_u8(b'e')
}

fn encode_list<B: AsRef<[u8]> + Ord>(vs: &Vec<Value<B>>, buf: &mut BytesMut) {
    encode_list_begin(buf);
    for v in vs {
        encode_value(v, buf);
    }
    encode_list_end(buf);
}

// d<(<str><value>)*>e
#[inline]
fn parse_dict(input: &[u8]) -> IResult<&[u8], Value<&[u8]>> {
    delimited(
        char('d'),
        map(
            fold_many0(
                tuple((parse_bytes, parse_value)),
                BTreeMap::new,
                |mut acc, (k, v)| {
                    acc.insert(k, v);
                    acc
                },
            ),
            |ps| Value::Dict(ps),
        ),
        char('e'),
    )(input)
}

#[inline]
fn encode_dict_begin(buf: &mut BytesMut) {
    buf.put_u8(b'd');
}

#[inline]
fn encode_dict_end(buf: &mut BytesMut) {
    buf.put_u8(b'e');
}

fn encode_dict<B: AsRef<[u8]> + Ord>(vs: &BTreeMap<B, Value<B>>, buf: &mut BytesMut) {
    encode_dict_begin(buf);
    for (k, v) in vs {
        encode_string(k, buf);
        encode_value(v, buf);
    }
    encode_dict_end(buf);
}

#[inline]
fn parse_value(input: &[u8]) -> IResult<&[u8], Value<&[u8]>> {
    alt((parse_bytes_value, parse_integer, parse_list, parse_dict))(input)
}

fn encode_value<B: AsRef<[u8]> + Ord>(value: &Value<B>, buf: &mut BytesMut) {
    match value {
        Value::Bytes(b) => encode_string(&b, buf),
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
    let result = all_consuming(terminated(parse_value, opt(char('\n'))))(input);
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
