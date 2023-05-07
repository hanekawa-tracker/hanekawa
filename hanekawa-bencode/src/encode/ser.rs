use crate::{Element, Elements};

use std::cell::RefCell;

use super::{
    encode_dict_begin, encode_dict_end, encode_integer, encode_list_begin, encode_list_end,
    encode_string, Value,
};

use bytes::{Bytes, BytesMut};
use serde::ser::{self, Impossible};

struct Serializer {
    buf: BytesMut,
    writing_map_key: bool,
}

impl Serializer {
    fn new() -> Self {
        Self {
            buf: BytesMut::new(),
            writing_map_key: false,
        }
    }

    fn reject_if_writing_map_key(&self) -> Result<(), Error> {
        if self.writing_map_key {
            Err(Error::InvalidMapKey)?
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    UnsupportedElement(Option<String>),
    InvalidMapKey,
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedElement(w) => {
                if let Some(w) = w {
                    f.write_fmt(format_args!("unsupported element: {}", w))
                } else {
                    f.write_str("unsupported element")
                }
            }
            Self::InvalidMapKey => f.write_str("invalid map key: keys must be strings"),
            Self::Other(s) => f.write_fmt(format_args!("other error: {}", s)),
        }
    }
}

fn unsupported_element<T>(which: Option<&str>) -> Result<T, Error> {
    Err(Error::UnsupportedElement(which.map(ToString::to_string)))
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Other(msg.to_string())
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = SeqSerializer<'a>;

    type SerializeTuple = Impossible<(), Error>;

    type SerializeTupleStruct = Impossible<(), Error>;

    type SerializeTupleVariant = Impossible<(), Error>;

    type SerializeMap = MapSerializer<'a>;

    type SerializeStruct = StructSerializer<'a>;

    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        unsupported_element(Some("bool"))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.reject_if_writing_map_key()?;

        encode_integer(v, &mut self.buf);

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        unsupported_element(Some("u64"))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        unsupported_element(Some("f32"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        unsupported_element(Some("f64"))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        unsupported_element(Some("char"))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        encode_string(v.as_bytes(), &mut self.buf);

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        encode_string(v, &mut self.buf);

        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.reject_if_writing_map_key()?;

        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.reject_if_writing_map_key()?;

        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        unsupported_element(Some("unit struct"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unsupported_element(Some("unit variant"))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.reject_if_writing_map_key()?;

        Ok(SeqSerializer::new(self))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unsupported_element(Some("tuple"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unsupported_element(Some("tuple struct"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unsupported_element(Some("tuple variant"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.reject_if_writing_map_key()?;

        Ok(MapSerializer::new(self))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.reject_if_writing_map_key()?;

        Ok(StructSerializer::new(self))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unsupported_element(Some("struct variant"))
    }
}

struct SeqSerializer<'a> {
    serializer: &'a mut Serializer,
}

impl<'a> SeqSerializer<'a> {
    fn new(serializer: &'a mut Serializer) -> Self {
        encode_list_begin(&mut serializer.buf);
        Self { serializer }
    }
}

impl<'a> ser::SerializeSeq for SeqSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        encode_list_end(&mut self.serializer.buf);
        Ok(())
    }
}

struct StructSerializer<'a> {
    serializer: &'a mut Serializer,
}

impl<'a> StructSerializer<'a> {
    fn new(serializer: &'a mut Serializer) -> Self {
        encode_dict_begin(&mut serializer.buf);
        Self { serializer }
    }
}

impl<'a> ser::SerializeStruct for StructSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        use serde::Serialize;

        key.serialize(&mut *self.serializer)?;
        value.serialize(&mut *self.serializer)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        encode_dict_end(&mut self.serializer.buf);

        Ok(())
    }
}

struct MapSerializer<'a> {
    serializer: &'a mut Serializer,
}

impl<'a> MapSerializer<'a> {
    fn new(serializer: &'a mut Serializer) -> Self {
        encode_dict_begin(&mut serializer.buf);
        Self { serializer }
    }
}

impl<'a> ser::SerializeMap for MapSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let current = self.serializer.writing_map_key;
        self.serializer.writing_map_key = true;

        key.serialize(&mut *self.serializer)?;

        self.serializer.writing_map_key = current;

        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        encode_dict_end(&mut self.serializer.buf);

        Ok(())
    }
}

impl<B: AsRef<[u8]> + Ord> serde::Serialize for Value<B> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Bytes(b) => serializer.serialize_bytes(b.as_ref()),
            Self::Int(i) => serializer.serialize_i64(*i),
            Self::List(vs) => {
                use serde::ser::SerializeSeq;

                let mut s = serializer.serialize_seq(Some(vs.len()))?;
                for v in vs {
                    s.serialize_element(v)?;
                }
                s.end()
            }
            Self::Dict(es) => {
                use serde::ser::SerializeMap;
                use serde_bytes::Bytes;

                let mut s = serializer.serialize_map(Some(es.len()))?;
                for (k, v) in es {
                    // Serialize the key as Bytes, not a seq of u8.
                    s.serialize_key(&Bytes::new(k.as_ref()))?;
                    s.serialize_value(v)?;
                }
                s.end()
            }
        }
    }
}

struct IterWrap<I>(RefCell<I>);

impl<'i, B: AsRef<[u8]> + Ord + 'i, I: Iterator<Item = &'i Element<B>>> serde::Serialize
    for IterWrap<I>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let next = { self.0.borrow_mut().next() };
        match next {
            Some(Element::Bytes(b)) => serializer.serialize_bytes(b.as_ref()),
            Some(Element::Int(i)) => serializer.serialize_i64(*i),
            Some(Element::ListBegin(ct)) => {
                use serde::ser::SerializeSeq;

                let mut s = serializer.serialize_seq(Some(*ct))?;
                for _ in 0..*ct {
                    s.serialize_element(self)?
                }
                s.end()
            }
            Some(Element::DictBegin(ct)) => {
                use serde::ser::SerializeMap;
                use serde_bytes::Bytes;

                let mut s = serializer.serialize_map(Some(*ct))?;
                for _ in 0..*ct {
                    let next = { self.0.borrow_mut().next() };
                    if let Some(Element::Bytes(k)) = next {
                        s.serialize_key(&Bytes::new(k.as_ref()))?;
                        s.serialize_value(self)?;
                    }
                }
                s.end()
            }
            _ => serializer.serialize_none(),
        }
    }
}

impl<B: AsRef<[u8]> + Ord> serde::Serialize for Elements<B> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let iter = self.into_iter();
        let mut wrap = IterWrap(RefCell::new(iter));
        let wrap = &mut wrap;

        wrap.serialize(serializer)
    }
}

pub fn to_bytes<T: serde::Serialize>(value: &T) -> Result<Bytes, Error> {
    let mut serializer = Serializer::new();
    value.serialize(&mut serializer)?;

    Ok(serializer.buf.into())
}

#[cfg(test)]
mod test {
    use super::*;
    use include_dir::{include_dir, Dir};

    static TORRENT_SAMPLES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/benches/samples/");

    #[test]
    fn encode_parsed_torrents() {
        for sample in TORRENT_SAMPLES_DIR.files() {
            let parsed = crate::parse(sample.contents()).expect(&format!(
                "failed to parse sample file: {}",
                sample.path().file_name().unwrap().to_string_lossy()
            ));

            let encoded = to_bytes(&parsed);

            assert_eq!(sample.contents(), encoded.unwrap());
        }
    }
}
