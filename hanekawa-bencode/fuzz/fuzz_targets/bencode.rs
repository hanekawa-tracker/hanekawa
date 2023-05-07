#![no_main]
use libfuzzer_sys::fuzz_target;

use hanekawa_bencode::{Value, parse, encode};

fuzz_target!(|input: Value<&[u8]>| {
    let expected = Ok(input.clone().into_elements());
    let encoded = encode(&input);
    let parsed_val = parse(&encoded);
    assert_eq!(expected, parsed_val);
});
