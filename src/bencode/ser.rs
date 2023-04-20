use super::{encode_dict, encode_integer, encode_list, encode_string, Value};

use bytes::{Bytes, BytesMut};
use serde::ser::{self, Impossible};
use std::collections::BTreeMap;

struct Serializer {
    buf: BytesMut,
    last_value: Option<Value<Vec<u8>>>,
    ignoring_buffer: bool,
}

impl Serializer {
    fn new() -> Self {
        Self {
            buf: BytesMut::new(),
            last_value: None,
            ignoring_buffer: false,
        }
    }

    fn without_buffering<T>(&mut self, action: impl Fn(&mut Self) -> T) -> T {
        let current = self.ignoring_buffer;

        self.ignoring_buffer = true;
        let t = action(self);
        self.ignoring_buffer = current;

        t
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
        if !self.ignoring_buffer {
            encode_integer(v, &mut self.buf);
        }
        self.last_value = Some(Value::Int(v));

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
        if !self.ignoring_buffer {
            encode_string(v.as_bytes(), &mut self.buf);
        }
        self.last_value = Some(Value::Bytes(v.as_bytes().to_vec()));

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if !self.ignoring_buffer {
            encode_string(v, &mut self.buf);
        }
        self.last_value = Some(Value::Bytes(v.to_vec()));

        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
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

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SeqSerializer::new(len, self))
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
        Ok(MapSerializer::new(self))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
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
    seq: Vec<Value<Vec<u8>>>,
    serializer: &'a mut Serializer,
}

impl<'a> SeqSerializer<'a> {
    fn new(len: Option<usize>, serializer: &'a mut Serializer) -> Self {
        Self {
            seq: Vec::with_capacity(len.unwrap_or(0)),
            serializer,
        }
    }
}

impl<'a> ser::SerializeSeq for SeqSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.serializer.without_buffering(|s| value.serialize(s))?;
        if let Some(v) = self.serializer.last_value.take() {
            self.seq.push(v);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if !self.serializer.ignoring_buffer {
            encode_list(&self.seq, &mut self.serializer.buf);
        }
        self.serializer.last_value = Some(Value::List(self.seq));

        Ok(())
    }
}

struct StructSerializer<'a> {
    entries: BTreeMap<Vec<u8>, Value<Vec<u8>>>,
    serializer: &'a mut Serializer,
}

impl<'a> StructSerializer<'a> {
    fn new(serializer: &'a mut Serializer) -> Self {
        Self {
            entries: BTreeMap::new(),
            serializer,
        }
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
        self.serializer.without_buffering(|s| value.serialize(s))?;

        if let Some(v) = self.serializer.last_value.take() {
            self.entries.insert(key.as_bytes().to_vec(), v);
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if !self.serializer.ignoring_buffer {
            encode_dict(&self.entries, &mut self.serializer.buf);
        }

        self.serializer.last_value = Some(Value::Dict(self.entries));

        Ok(())
    }
}

struct MapSerializer<'a> {
    entries: BTreeMap<Vec<u8>, Value<Vec<u8>>>,
    current_key: Option<Vec<u8>>,
    serializer: &'a mut Serializer,
}

impl<'a> MapSerializer<'a> {
    fn new(serializer: &'a mut Serializer) -> Self {
        Self {
            entries: BTreeMap::new(),
            current_key: None,
            serializer,
        }
    }
}

impl<'a> ser::SerializeMap for MapSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.serializer.without_buffering(|s| key.serialize(s))?;
        if let Some(Value::Bytes(s)) = self.serializer.last_value.take() {
            self.current_key = Some(s);
            Ok(())
        } else {
            Err(Error::InvalidMapKey)
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.serializer.without_buffering(|s| value.serialize(s))?;

        if let Some(v) = self.serializer.last_value.take() {
            if let Some(k) = self.current_key.take() {
                self.entries.insert(k, v);
            }
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if !self.serializer.ignoring_buffer {
            encode_dict(&self.entries, &mut self.serializer.buf);
        }
        self.serializer.last_value = Some(Value::Dict(self.entries));

        Ok(())
    }
}

impl<B: AsRef<[u8]> + Ord> Value<B> {
    fn into_serializable(self) -> impl serde::Serialize {
        use serde_bytes::ByteBuf;

        #[derive(serde::Serialize)]
        #[serde(untagged)]
        enum SerValue {
            Bytes(ByteBuf),
            Int(i64),
            List(Vec<Self>),
            Dict(BTreeMap<ByteBuf, Self>),
        }

        impl<B: AsRef<[u8]> + Ord> Into<SerValue> for Value<B> {
            fn into(self) -> SerValue {
                match self {
                    Self::Bytes(bs) => SerValue::Bytes(ByteBuf::from(bs.as_ref().to_vec())),
                    Self::Int(i) => SerValue::Int(i),
                    Self::List(ls) => SerValue::List(ls.into_iter().map(Into::into).collect()),
                    Self::Dict(d) => {
                        let map = d
                            .into_iter()
                            .map(|(k, v)| (ByteBuf::from(k.as_ref().to_vec()), v.into()))
                            .collect();
                        SerValue::Dict(map)
                    }
                }
            }
        }

        let sv: SerValue = self.into();
        sv
    }
}

pub fn value_to_bytes<B: AsRef<[u8]> + Ord>(value: Value<B>) -> Result<Bytes, Error> {
    let ser = value.into_serializable();
    to_bytes(&ser)
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
            let parsed = crate::bencode::parse(sample.contents()).expect(&format!(
                "failed to parse sample file: {}",
                sample.path().file_name().unwrap().to_string_lossy()
            ));

            let encoded = value_to_bytes(parsed);

            assert!(encoded.is_ok());
        }
    }
}
