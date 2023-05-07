mod decode;
mod encode;
mod map;
mod repr;

pub use decode::parse;
pub use encode::encode;
pub use encode::ser::to_bytes;

pub use map::Map;
pub use repr::{Element, Elements, Error, Value};

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
