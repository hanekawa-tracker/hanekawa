mod map;
mod ser;

pub use ser::to_bytes;

use bytes::{BufMut, BytesMut};

use nom::{
    branch::alt,
    character::complete::{char, digit1},
    combinator::{all_consuming, map, opt, recognize, verify},
    multi::{fold_many0, many0},
    sequence::{delimited, terminated, tuple},
    IResult,
};

use map::Map;

#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value<B: Ord> {
    Bytes(B),
    Int(i64),
    List(Vec<Self>),
    Dict(Map<B, Self>),
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

#[inline(always)]
fn encode_list_begin(buf: &mut BytesMut) {
    buf.put_u8(b'l');
}

#[inline(always)]
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
                || Map::with_capacity(4),
                |mut acc, (k, v)| {
                    acc.insert(k, v);
                    acc
                },
            ),
            |mut ps| {
                ps.ensure_order();

                Value::Dict(ps)
            },
        ),
        char('e'),
    )(input)
}

#[inline(always)]
fn encode_dict_begin(buf: &mut BytesMut) {
    buf.put_u8(b'd');
}

#[inline(always)]
fn encode_dict_end(buf: &mut BytesMut) {
    buf.put_u8(b'e');
}

fn encode_dict<B: AsRef<[u8]> + Ord>(vs: &Map<B, Value<B>>, buf: &mut BytesMut) {
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

pub fn rd_parse(input: &[u8]) -> Result<Value<&[u8]>, ()> {
    let parser = rd::Parser::new(input);
    parser.parse().map_err(|_| ())
}

mod rd {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    pub enum Error {
        UnexpectedEnd(String),
        ExpectedFar(u8),
        InvalidInt(Vec<u8>),
        Trailing,
    }

    struct ParseBufs<'a> {
        list: Vec<Value<&'a [u8]>>,
        map: Vec<(&'a [u8], Value<&'a [u8]>)>,
    }

    impl<'a> ParseBufs<'a> {
        fn new() -> Self {
            Self {
                list: Vec::with_capacity(10),
                map: Vec::with_capacity(10),
            }
        }
    }

    pub struct Parser<'a> {
        input: &'a [u8],
        bufs: ParseBufs<'a>,
    }

    impl<'a> Parser<'a> {
        pub fn new(input: &'a [u8]) -> Self {
            let bufs = ParseBufs::new();
            Self { input, bufs }
        }

        #[inline(always)]
        fn is_done(&self) -> bool {
            self.input.len() == 0
        }

        #[inline(always)]
        fn peek(&self) -> Option<u8> {
            self.input.get(0).copied()
        }

        #[inline(always)]
        fn take_until(&mut self, b: u8) -> Result<&'a [u8], Error> {
            let result = memchr::memchr(b, &self.input);
            match result {
                Some(i) => {
                    let (head, tail) = self.input.split_at(i);
                    self.input = tail;
                    Ok(head)
                }
                _ => Err(Error::ExpectedFar(b)),
            }
        }

        #[inline(always)]
        fn take_n(&mut self, n: usize) -> Result<&'a [u8], Error> {
            let (head, tail) = self.input.split_at(n);
            if head.len() == n {
                self.input = tail;
                Ok(head)
            } else {
                Err(Error::UnexpectedEnd(format!("take_n: {}", n)))
            }
        }

        #[inline(always)]
        fn bump(&mut self) -> Result<(), Error> {
            match self.input.split_first() {
                Some((_, tail)) => {
                    self.input = tail;
                    Ok(())
                }
                _ => Err(Error::UnexpectedEnd("bump".to_string())),
            }
        }

        #[inline(always)]
        fn bump_assert(&mut self) {
            let result = self.bump();
            debug_assert!(result.is_ok());
        }

        #[inline(always)]
        fn parse_raw_int<T: lexical::FromLexical>(input: &[u8]) -> Result<T, Error> {
            lexical::parse(input).map_err(|_| Error::InvalidInt(input.to_vec()))
        }

        fn parse_string(&mut self) -> Result<&'a [u8], Error> {
            let len = self.take_until(b':')?;
            self.bump()?;
            let len_num = Self::parse_raw_int(len)?;
            let str = self.take_n(len_num)?;

            Ok(str)
        }

        fn parse_dict(&mut self) -> Result<Value<&'a [u8]>, Error> {
            self.bump_assert();

            let start_len = self.bufs.map.len();
            while self.peek() != Some(b'e') {
                let key = self.parse_string()?;
                let value = self.parse_value()?;
                self.bufs.map.push((key, value));
            }

            self.bump_assert();

            let map = Map::from_raw(self.bufs.map.split_off(start_len));

            Ok(Value::Dict(map))
        }

        fn parse_int(&mut self) -> Result<Value<&'a [u8]>, Error> {
            self.bump_assert();

            let num = self.take_until(b'e')?;
            // Reject leading -0 and leading 0, but not 0 itself.
            if num.starts_with(&[b'-', b'0']) || (num.starts_with(&[b'0']) && num.len() != 1) {
                Err(Error::InvalidInt(num.to_vec()))?;
            }
            let num = lexical::parse(&num).unwrap();

            self.bump_assert();

            Ok(Value::Int(num))
        }

        fn parse_list(&mut self) -> Result<Value<&'a [u8]>, Error> {
            self.bump_assert();

            let start_len = self.bufs.list.len();
            while self.peek() != Some(b'e') {
                let value = self.parse_value()?;
                self.bufs.list.push(value);
            }

            let list = self.bufs.list.split_off(start_len);

            self.bump_assert();

            Ok(Value::List(list))
        }

        fn parse_value(&mut self) -> Result<Value<&'a [u8]>, Error> {
            match self.peek() {
                Some(b'd') => self.parse_dict(),
                Some(b'i') => self.parse_int(),
                Some(b'l') => self.parse_list(),
                Some(b'0'..=b'9') => Ok(Value::Bytes(self.parse_string()?)),
                _ => Err(Error::UnexpectedEnd("parse_value".to_string())),
            }
        }

        pub fn parse(mut self) -> Result<Value<&'a [u8]>, Error> {
            let value = self.parse_value()?;
            if self.is_done() {
                Ok(value)
            } else {
                Err(Error::Trailing)
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn parses_string() {
            let enc = "4:spam".as_bytes();
            assert_eq!(Value::Bytes("spam".as_bytes()), rd_parse(&enc).unwrap())
        }

        #[test]
        fn parses_valid_ints() {
            assert_eq!(Value::Int(3), rd_parse("i3e".as_bytes()).unwrap());
            assert_eq!(Value::Int(0), rd_parse("i0e".as_bytes()).unwrap())
        }

        #[test]
        fn rejects_invalid_ints() {
            assert!(
                rd_parse("i03e".as_bytes()).is_err(),
                "leading zeros are invalid"
            );
            assert!(
                rd_parse("i-0e".as_bytes()).is_err(),
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
                rd_parse(enc).unwrap()
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
                rd_parse("d3:cow3:moo4:spam4:eggse".as_bytes()).unwrap()
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
                rd_parse("d4:spaml1:a1:bee".as_bytes()).unwrap()
            );
        }
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
