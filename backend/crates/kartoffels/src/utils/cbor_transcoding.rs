use ciborium::Value as CborValue;
use serde_json::{Number as JsonNumber, Value as JsonValue};

pub fn cbor_to_json(val: CborValue) -> JsonValue {
    match val {
        CborValue::Integer(val) => {
            JsonValue::Number((i128::from(val) as i64).into())
        }

        CborValue::Bytes(_) => JsonValue::String("[bytes]".into()),

        CborValue::Float(val) => {
            JsonValue::Number(JsonNumber::from_f64(val).unwrap())
        }

        CborValue::Text(val) => JsonValue::String(val),
        CborValue::Bool(val) => JsonValue::Bool(val),
        CborValue::Null => JsonValue::Null,

        CborValue::Array(val) => {
            JsonValue::Array(val.into_iter().map(cbor_to_json).collect())
        }

        CborValue::Map(val) => JsonValue::Object(
            val.into_iter()
                .map(|(key, val)| {
                    let JsonValue::String(key) = cbor_to_json(key) else {
                        panic!();
                    };

                    let val = cbor_to_json(val);

                    (key, val)
                })
                .collect(),
        ),

        val => panic!("unexpected value: {val:?}"),
    }
}

pub fn json_to_cbor(val: JsonValue) -> CborValue {
    match val {
        JsonValue::Null => CborValue::Null,
        JsonValue::Bool(val) => CborValue::Bool(val),

        JsonValue::Number(val) => {
            if let Some(val) = val.as_i64() {
                CborValue::Integer((val as i128).try_into().unwrap())
            } else {
                CborValue::Float(val.as_f64().unwrap())
            }
        }

        JsonValue::String(val) => CborValue::Text(val),

        JsonValue::Array(val) => {
            CborValue::Array(val.into_iter().map(json_to_cbor).collect())
        }

        JsonValue::Object(val) => CborValue::Map(
            val.into_iter()
                .map(|(key, val)| {
                    let key = CborValue::Text(key);
                    let val = json_to_cbor(val);

                    (key, val)
                })
                .collect(),
        ),
    }
}
