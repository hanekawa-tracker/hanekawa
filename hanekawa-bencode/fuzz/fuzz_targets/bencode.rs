#![no_main]
use libfuzzer_sys::fuzz_target;

use hanekawa_bencode::{Value, parse, encode};

fuzz_target!(|input: Value<&[u8]>| {
    let expected = Ok(input.clone());
    let encoded = encode(&input);
    assert_eq!(expected, parse(&encoded));
});
