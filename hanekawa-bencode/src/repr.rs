use crate::map::Map;

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

impl<B> Elements<B> {
    pub(crate) fn from_parts(elements: Vec<Element<B>>) -> Self {
        Self { elements }
    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn converts_strings_to_value() {
        let els = Elements::from_parts(vec![Element::Bytes("spam".as_bytes())]);
        let exp = Value::Bytes("spam".as_bytes());
        let value = els.into_value();
        assert_eq!(exp, value);
    }

    #[test]
    fn converts_ints_to_value() {
        let els = Elements::from_parts(vec![Element::<&[u8]>::Int(127)]);
        let exp = Value::Int(127);
        let value = els.into_value();
        assert_eq!(exp, value);
    }

    #[test]
    fn converts_lists_to_value() {
        let els = Elements::from_parts(vec![
            Element::ListBegin(2),
            Element::Bytes("spam".as_bytes()),
            Element::Int(127),
        ]);
        let exp = Value::List(vec![Value::Bytes("spam".as_bytes()), Value::Int(127)]);
        let value = els.into_value();
        assert_eq!(exp, value);
    }

    #[test]
    fn converts_dicts_to_value() {
        let els = Elements::from_parts(vec![
            Element::DictBegin(2),
            Element::Bytes("values".as_bytes()),
            Element::ListBegin(2),
            Element::Bytes("spam".as_bytes()),
            Element::Int(127),
            Element::Bytes("key".as_bytes()),
            Element::Bytes("value".as_bytes()),
        ]);

        let mut dict = Map::new();
        dict.insert(
            "values".as_bytes(),
            Value::List(vec![Value::Bytes("spam".as_bytes()), Value::Int(127)]),
        );
        dict.insert("key".as_bytes(), Value::Bytes("value".as_bytes()));

        let exp = Value::Dict(dict);
        let value = els.into_value();

        assert_eq!(exp, value);
    }
}
