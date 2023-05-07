use super::{Element, Elements, Error};

pub fn parse(input: &[u8]) -> Result<Elements<&[u8]>, ()> {
    let parser = Parser::new(input);
    parser.parse().map_err(|_| ())
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
            Ok(Elements::from_parts(self.elements))
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
        let exp = Elements::from_parts(vec![Element::Bytes("spam".as_bytes())]);
        assert_eq!(exp, parse(&enc).unwrap())
    }

    #[test]
    fn parses_valid_ints() {
        assert_eq!(
            Elements::from_parts(vec![Element::Int(3)]),
            parse("i3e".as_bytes()).unwrap()
        );
        assert_eq!(
            Elements::from_parts(vec![Element::Int(0)]),
            parse("i0e".as_bytes()).unwrap()
        )
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
            Elements::from_parts(vec![
                Element::ListBegin(2),
                Element::Bytes("spam".as_bytes()),
                Element::Bytes("eggs".as_bytes())
            ]),
            parse(enc).unwrap()
        )
    }

    #[test]
    fn parses_dicts() {
        assert_eq!(
            Elements::from_parts(vec![
                Element::DictBegin(2),
                Element::Bytes("cow".as_bytes()),
                Element::Bytes("moo".as_bytes()),
                Element::Bytes("spam".as_bytes()),
                Element::Bytes("eggs".as_bytes())
            ]),
            parse("d3:cow3:moo4:spam4:eggse".as_bytes()).unwrap()
        );

        assert_eq!(
            Elements::from_parts(vec![
                Element::DictBegin(1),
                Element::Bytes("spam".as_bytes()),
                Element::ListBegin(2),
                Element::Bytes("a".as_bytes()),
                Element::Bytes("b".as_bytes())
            ]),
            parse("d4:spaml1:a1:bee".as_bytes()).unwrap()
        );
    }
}
