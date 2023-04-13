#![no_main]
use libfuzzer_sys::fuzz_target;

use hanekawa::bencode::{Value, parse, encode};

fuzz_target!(|input: Value| {
    let expected = Ok(input.clone());
    let encoded = encode(&input);
    assert_eq!(expected, parse(&encoded));
});
