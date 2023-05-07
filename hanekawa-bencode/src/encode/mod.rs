pub mod ser;

use super::{Map, Value};
use bytes::{BufMut, BytesMut};

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
