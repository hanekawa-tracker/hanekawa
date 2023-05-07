mod map;
mod ser;

pub use ser::to_bytes;

use bytes::{BufMut, BytesMut};

use map::Map;

#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value<B: Ord> {
    Bytes(B),
    Int(i64),
    List(Vec<Self>),
    Dict(Map<B, Self>),
}

impl<B: Ord> Value<B> {
    pub fn into_elements(self) -> Elements<B> {
        pub struct IntoElements<B> {
            elements: Vec<Element<B>>,
        }

        impl<B: Ord> IntoElements<B> {
            fn into_elements(&mut self, value: Value<B>) {
                match value {
                    Value::Bytes(bs) => self.elements.push(Element::Bytes(bs)),
                    Value::Int(i) => self.elements.push(Element::Int(i)),
                    Value::List(vs) => {
                        self.elements.push(Element::ListBegin(vs.len()));
                        for v in vs {
                            self.into_elements(v)
                        }
                    }
                    Value::Dict(m) => {
                        self.elements.push(Element::DictBegin(m.len()));
                        for (k, v) in m {
                            self.elements.push(Element::Bytes(k));
                            self.into_elements(v);
                        }
                    }
                }
            }
        }

        let mut ii = IntoElements {
            elements: Vec::with_capacity(10),
        };
        ii.into_elements(self);
        Elements {
            elements: ii.elements,
        }
    }
}

fn encode_string<B: AsRef<[u8]> + Ord>(bytes: B, buf: &mut BytesMut) {
    use lexical::{FormattedSize, ToLexical};
    let mut digits = [0; usize::FORMATTED_SIZE_DECIMAL];

    let bytes = bytes.as_ref();

    buf.put_slice(bytes.len().to_lexical(&mut digits));
    buf.put_u8(b':');
    buf.put_slice(&bytes);
}

fn encode_integer(i: i64, buf: &mut BytesMut) {
    use lexical::{FormattedSize, ToLexical};
    let mut digits = [0; usize::FORMATTED_SIZE_DECIMAL];

    buf.put_u8(b'i');
    buf.put_slice(i.to_lexical(&mut digits));
    buf.put_u8(b'e');
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

pub fn parse(input: &[u8]) -> Result<Elements<&[u8]>, ()> {
    let parser = Parser::new(input);
    parser.parse().map_err(|_| ())
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UnexpectedEnd(String),
    ExpectedFar(u8),
    InvalidInt(Vec<u8>),
    Trailing,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Element<B> {
    DictBegin(usize),
    ListBegin(usize),
    Int(i64),
    Bytes(B),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Elements<B> {
    elements: Vec<Element<B>>,
}

impl<'a, B> IntoIterator for &'a Elements<B> {
    type Item = &'a Element<B>;

    type IntoIter = <&'a Vec<Element<B>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let b: &Vec<Element<B>> = &self.elements;
        b.into_iter()
    }
}

impl<B: Ord + AsRef<[u8]>> Elements<B> {
    pub fn into_value(self) -> Value<B> {
        struct IntoValue<B> {
            drain: <Vec<Element<B>> as IntoIterator>::IntoIter,
        }

        impl<B: Ord + AsRef<[u8]>> IntoValue<B> {
            fn into_value(&mut self) -> Value<B> {
                match self.drain.next() {
                    Some(Element::Int(i)) => Value::Int(i),
                    Some(Element::Bytes(bs)) => Value::Bytes(bs),
                    Some(Element::ListBegin(ct)) => {
                        let mut list = Vec::with_capacity(ct);
                        for _ in 0..ct {
                            list.push(self.into_value());
                        }
                        Value::List(list)
                    }
                    Some(Element::DictBegin(ct)) => {
                        let mut map = Map::with_capacity(ct);
                        for _ in 0..ct {
                            let key = self.into_value();
                            let value = self.into_value();
                            if let Value::Bytes(b) = key {
                                map.insert(b, value);
                            }
                        }
                        Value::Dict(map)
                    }
                    _ => unreachable!(),
                }
            }
        }

        let mut iv = IntoValue {
            drain: self.elements.into_iter(),
        };
        iv.into_value()
    }
}

pub struct Parser<'a> {
    input: &'a [u8],
    elements: Vec<Element<&'a [u8]>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        let elements = Vec::with_capacity(10);
        Self { input, elements }
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
    fn bump(&mut self) -> Result<u8, Error> {
        match self.input.split_first() {
            Some((t, tail)) => {
                self.input = tail;
                Ok(*t)
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

    fn parse_string(&mut self) -> Result<(), Error> {
        let len = self.take_until(b':')?;
        self.bump_assert();
        let len_num = Self::parse_raw_int(len)?;
        let str = self.take_n(len_num)?;

        self.elements.push(Element::Bytes(str));

        Ok(())
    }

    fn parse_dict(&mut self) -> Result<(), Error> {
        self.bump_assert();

        let header_idx = self.elements.len();
        self.elements.push(Element::DictBegin(0));

        let mut ct = 0;

        while self.peek() != Some(b'e') {
            self.parse_string()?;
            self.parse_value()?;
            ct += 1;
        }

        self.bump_assert();

        self.elements[header_idx] = Element::DictBegin(ct);

        Ok(())
    }

    fn parse_int(&mut self) -> Result<(), Error> {
        self.bump_assert();

        let num = self.take_until(b'e')?;
        // Reject leading -0 and leading 0, but not 0 itself.
        if num.starts_with(&[b'-', b'0']) || (num.starts_with(&[b'0']) && num.len() != 1) {
            Err(Error::InvalidInt(num.to_vec()))?;
        }
        let num = lexical::parse(&num).unwrap();

        self.bump_assert();

        self.elements.push(Element::Int(num));

        Ok(())
    }

    fn parse_list(&mut self) -> Result<(), Error> {
        self.bump_assert();

        let header_idx = self.elements.len();

        self.elements.push(Element::ListBegin(0));

        let mut ct = 0;

        while self.peek() != Some(b'e') {
            self.parse_value()?;
            ct += 1;
        }

        self.bump_assert();

        self.elements[header_idx] = Element::ListBegin(ct);

        Ok(())
    }

    fn parse_value(&mut self) -> Result<(), Error> {
        match self.peek() {
            Some(b'd') => self.parse_dict(),
            Some(b'i') => self.parse_int(),
            Some(b'l') => self.parse_list(),
            Some(b'0'..=b'9') => self.parse_string(),
            _ => Err(Error::UnexpectedEnd("parse_value".to_string())),
        }
    }

    pub fn parse(mut self) -> Result<Elements<&'a [u8]>, Error> {
        self.parse_value()?;
        if self.is_done() {
            Ok(Elements {
                elements: self.elements,
            })
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
        assert_eq!(
            Value::Bytes("spam".as_bytes()),
            parse(&enc).unwrap().into_value()
        )
    }

    #[test]
    fn parses_valid_ints() {
        assert_eq!(Value::Int(3), parse("i3e".as_bytes()).unwrap().into_value());
        assert_eq!(Value::Int(0), parse("i0e".as_bytes()).unwrap().into_value())
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
            parse(enc).unwrap().into_value()
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
            parse("d3:cow3:moo4:spam4:eggse".as_bytes())
                .unwrap()
                .into_value()
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
            parse("d4:spaml1:a1:bee".as_bytes()).unwrap().into_value()
        );
    }

    #[test]
    fn converts_strings_to_value() {
        let data = "4:spam";
        let els = parse(data.as_bytes()).unwrap();
        let value = els.into_value();
        assert_eq!(Value::Bytes("spam".as_bytes()), value);
    }

    #[test]
    fn converts_ints_to_value() {
        let data = "i127e";
        let els = parse(data.as_bytes()).unwrap();
        let value = els.into_value();
        assert_eq!(Value::Int(127), value);
    }

    #[test]
    fn converts_lists_to_value() {
        let data = "l4:spami127ee";
        let els = parse(data.as_bytes()).unwrap();
        let value = els.into_value();
        assert_eq!(
            Value::List(vec![Value::Bytes("spam".as_bytes()), Value::Int(127)]),
            value
        );
    }

    #[test]
    fn converts_dicts_to_value() {
        let data = "d6:valuesl4:spami127ee3:key5:valuee";
        let els = parse(data.as_bytes()).unwrap();
        let value = els.into_value();
        let mut dict = Map::new();
        dict.insert(
            "values".as_bytes(),
            Value::List(vec![Value::Bytes("spam".as_bytes()), Value::Int(127)]),
        );
        dict.insert("key".as_bytes(), Value::Bytes("value".as_bytes()));
        assert_eq!(Value::Dict(dict), value);
    }
}
