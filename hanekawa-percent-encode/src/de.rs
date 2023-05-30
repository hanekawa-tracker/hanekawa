use serde::de::IntoDeserializer;
use serde::de::{
    value::{self, MapDeserializer, SeqDeserializer},
    Error,
};
use serde::forward_to_deserialize_any;

use std::borrow::Cow;

#[derive(Debug)]
enum Value<'a> {
    Bytes(Cow<'a, [u8]>),
    Multi(Vec<Cow<'a, [u8]>>),
}

impl<'a> Value<'a> {
    fn bytes(self) -> Cow<'a, [u8]> {
        match self {
            Self::Bytes(bs) => bs,
            Self::Multi(mut vs) => {
                debug_assert!(!vs.is_empty());
                let head = vs.remove(0);
                head
            }
        }
    }

    fn str(self) -> Cow<'a, str> {
        match self.bytes() {
            Cow::Borrowed(bs) => String::from_utf8_lossy(bs),
            Cow::Owned(vs) => match String::from_utf8_lossy(&vs) {
                Cow::Borrowed(bs) => Cow::Owned(bs.to_string()),
                Cow::Owned(o) => Cow::Owned(o),
            },
        }
    }
}

struct Parts<'a>(Vec<(Cow<'a, str>, Value<'a>)>);

impl<'a> Parts<'a> {
    fn from_query_string(query_string: &'a str) -> Self {
        use std::collections::HashMap;

        let mut map = HashMap::new();

        let parts = query_string.split('&').map(|param| {
            let (key, value) = param.split_once('=').unwrap();
            let key = percent_encoding::percent_decode_str(key).decode_utf8_lossy();
            let value: Cow<'_, [u8]> = percent_encoding::percent_decode_str(value).into();
            (key, value)
        });

        for (key, value) in parts.into_iter() {
            use std::collections::hash_map::Entry;

            let entry = map.entry(key);
            match entry {
                Entry::Occupied(mut e) => {
                    match e.get_mut() {
                        Value::Bytes(bs) => {
                            let old = std::mem::replace(bs, Cow::Borrowed(&[]));
                            e.insert(Value::Multi(vec![old, value]));
                        }
                        Value::Multi(ref mut vs) => {
                            vs.push(value);
                        }
                    };
                }
                Entry::Vacant(e) => {
                    e.insert(Value::Bytes(value));
                }
            };
        }

        Self(map.into_iter().collect())
    }
}

impl<'a> IntoIterator for Parts<'a> {
    type Item = (Cow<'a, str>, Value<'a>);

    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub struct Deserializer<'de> {
    inner: MapDeserializer<'de, <Parts<'de> as IntoIterator>::IntoIter, value::Error>,
}

impl<'de> Deserializer<'de> {
    fn new(parts: Parts<'de>) -> Self {
        Self {
            inner: serde::de::value::MapDeserializer::new(parts.into_iter()),
        }
    }
}

impl<'de> serde::de::Deserializer<'de> for Deserializer<'de> {
    type Error = serde::de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.inner.end()?;
        visitor.visit_unit()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(self.inner)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(self.inner)
    }

    forward_to_deserialize_any! {
        bool
        u8
        u16
        u32
        u64
        i8
        i16
        i32
        i64
        f32
        f64
        char
        str
        string
        option
        bytes
        byte_buf
        unit_struct
        newtype_struct
        tuple_struct
        struct
        identifier
        tuple
        enum
        ignored_any
    }
}

impl<'de> serde::de::IntoDeserializer<'de> for Value<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

macro_rules! parse_from_str {
    ($($ty:ident => $method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where V: serde::de::Visitor<'de>
            {
                use serde::de::IntoDeserializer;

                let s = self.str();

                match s.parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$method(visitor),
                    Err(e) => Err(Error::custom(e))
                }
            }
        )*
    }
}

impl<'de> serde::de::Deserializer<'de> for Value<'de> {
    type Error = value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let s = self.str();

        match s {
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            Cow::Owned(s) => visitor.visit_string(s),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let bs = self.bytes();
        visitor.visit_bytes(&bs)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let bs = self.bytes();
        visitor.visit_bytes(&bs)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if let Value::Multi(vs) = self {
            let sd = SeqDeserializer::new(vs.into_iter().map(|v| Value::Bytes(v)));
            visitor.visit_seq(sd)
        } else {
            Err(Error::custom("expected sequence"))
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let s: String = self.str().to_string();
        visitor.visit_enum(EnumAccess(s))
    }

    forward_to_deserialize_any! {
        char
        str
        string
        unit
        unit_struct
        tuple_struct
        struct
        identifier
        tuple
        ignored_any
        map
    }

    parse_from_str! {
        bool => deserialize_bool,
        u8 => deserialize_u8,
        u16 => deserialize_u16,
        u32 => deserialize_u32,
        u64 => deserialize_u64,
        i8 => deserialize_i8,
        i16 => deserialize_i16,
        i32 => deserialize_i32,
        i64 => deserialize_i64,
        f32 => deserialize_f32,
        f64 => deserialize_f64,
    }
}

struct VariantAccess;

impl<'de> serde::de::VariantAccess<'de> for VariantAccess {
    type Error = value::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        Err(value::Error::custom("expected unit variant"))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(value::Error::custom("expected unit variant"))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(value::Error::custom("expected unit variant"))
    }
}

struct EnumAccess(String);

impl<'de> serde::de::EnumAccess<'de> for EnumAccess {
    type Error = value::Error;

    type Variant = VariantAccess;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let v = seed.deserialize(self.0.into_deserializer())?;
        Ok((v, VariantAccess))
    }
}

pub fn from_query_string<'q, T: serde::Deserialize<'q>>(query_string: &'q str) -> T {
    let parts = Parts::from_query_string(query_string);
    let deserializer = Deserializer::new(parts);
    T::deserialize(deserializer).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserializes() {
        #[derive(Debug, serde::Deserialize, PartialEq)]
        struct Person<'a> {
            name: &'a str,
            age: i32,
            #[serde(with = "serde_bytes")]
            serial_number: Vec<u8>,
            pet_names: Vec<String>,
            is_alive: bool,
        }

        let qs = "name=humanoid&age=1000&serial_number=%00%2042&pet_names=jim&is_alive=true&pet_names=tom";

        let value: Person = from_query_string(qs);

        assert_eq!(
            value,
            Person {
                name: "humanoid",
                age: 1000,
                serial_number: vec![b'\0', b' ', b'4', b'2'],
                pet_names: vec!["jim".to_string(), "tom".to_string()],
                is_alive: true
            }
        )
    }

    #[test]
    fn deserializes_non_unicode() {
        #[derive(Debug, serde::Deserialize, PartialEq)]
        struct Query {
            #[serde(with = "serde_bytes")]
            value: Vec<u8>
        }

        let query: Query = from_query_string("value=%26c%1d%91!");

        assert_eq!(
            query,
            Query {
                value: vec![
                    b'\x26',
                    b'c',
                    b'\x1d',
                    b'\x91',
                    b'!'
                ]
            }
        )
    }
}
