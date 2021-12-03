use super::{Attribute, Block, Body, Structure, Value};
use crate::value::Map;
use crate::Error;

impl TryFrom<Value> for Body {
    type Error = Error;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Array(array) => TryFrom::try_from(array),
            _ => Err(Error::new("array expected")),
        }
    }
}

impl TryFrom<Vec<Value>> for Body {
    type Error = Error;

    fn try_from(array: Vec<Value>) -> Result<Self, Self::Error> {
        array
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<Body, Self::Error>>()
    }
}

impl TryFrom<Value> for Structure {
    type Error = Error;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Object(object) => TryFrom::try_from(object),
            _ => Err(Error::new("object expected")),
        }
    }
}

impl TryFrom<Map<String, Value>> for Structure {
    type Error = Error;

    fn try_from(map: Map<String, Value>) -> Result<Self, Self::Error> {
        match map.get("kind") {
            Some(Value::String(kind)) => match kind.as_str() {
                "attribute" => Attribute::try_from(map).map(Structure::Attribute),
                "block" => Block::try_from(map).map(Structure::Block),
                kind => Err(Error::new(format!("invalid HCL structure kind `{}`", kind))),
            },
            _ => Err(Error::new("not a HCL structure")),
        }
    }
}

impl TryFrom<Map<String, Value>> for Attribute {
    type Error = Error;

    fn try_from(map: Map<String, Value>) -> Result<Self, Self::Error> {
        let key = map
            .get("key")
            .and_then(|i| i.as_str())
            .ok_or_else(|| Error::new("attribute key missing or not a string"))?;

        let value = map
            .get("value")
            .ok_or_else(|| Error::new("attribute value missing"))?;

        Ok(Attribute::new(key, value.clone()))
    }
}

impl TryFrom<Map<String, Value>> for Block {
    type Error = Error;

    fn try_from(map: Map<String, Value>) -> Result<Self, Self::Error> {
        let ident = map
            .get("ident")
            .and_then(|i| i.as_str())
            .ok_or_else(|| Error::new("block identifier missing or not a string"))?;

        let keys = match map.get("keys") {
            Some(Value::Array(array)) => {
                if array.iter().all(Value::is_string) {
                    array
                        .iter()
                        .filter_map(Value::as_str)
                        .map(|s| s.to_string())
                        .collect()
                } else {
                    return Err(Error::new("block keys must be strings"));
                }
            }
            Some(_) => return Err(Error::new("block keys must be an array")),
            None => Vec::new(),
        };

        let body = match map.get("body") {
            Some(Value::Array(array)) => Body::try_from(array.clone())?,
            _ => return Err(Error::new("block body missing or not an array")),
        };

        Ok(Block::new(ident, keys, body))
    }
}

impl From<Attribute> for Structure {
    fn from(attr: Attribute) -> Self {
        Structure::Attribute(attr)
    }
}

impl From<Block> for Structure {
    fn from(block: Block) -> Self {
        Structure::Block(block)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::value::Map;
    use maplit::hashmap;

    #[test]
    fn attribute_from_value() {
        let value = Value::Object(hashmap! {
            "kind".into() => "attribute".into(),
            "key".into() => "foo".into(),
            "value".into() => "bar".into()
        });

        assert_eq!(
            Structure::try_from(value).unwrap(),
            Structure::Attribute(Attribute::new("foo".into(), Value::String("bar".into())))
        );

        let value = Value::Object(Map::new());

        assert!(Structure::try_from(value).is_err());

        let value = Value::Object(hashmap! {
            "kind".into() => "attribute".into(),
            "key".into() => "foo".into(),
        });

        assert!(Structure::try_from(value).is_err());

        let value = Value::Object(hashmap! {
            "kind".into() => "attribute".into(),
            "value".into() => "bar".into()
        });

        assert!(Structure::try_from(value).is_err());
    }

    #[test]
    fn block_from_value() {
        let value = Value::Object(hashmap! {
            "kind".into() => "block".into(),
            "ident".into() => "resource".into(),
            "keys".into() => Value::Array(vec![
                "aws_s3_bucket".into(),
                "mybucket".into()
            ]),
            "body".into() => Value::Array(vec![
                Value::Object(hashmap! {
                    "kind".into() => "attribute".into(),
                    "key".into() => "name".into(),
                    "value".into() => "mybucket".into()
                })
            ])
        });

        assert_eq!(
            Structure::try_from(value).unwrap(),
            Structure::Block(Block::new(
                "resource",
                vec!["aws_s3_bucket".into(), "mybucket".into()],
                vec![Structure::Attribute(Attribute::new(
                    "name".into(),
                    Value::String("mybucket".into())
                ))]
            ))
        );

        let value = Value::Object(hashmap! {
            "kind".into() => "block".into(),
            "body".into() => Value::Array(vec![
                Value::Object(hashmap! {
                    "kind".into() => "attribute".into(),
                    "key".into() => "name".into(),
                    "value".into() => "mybucket".into()
                })
            ])
        });

        assert!(Structure::try_from(value).is_err());

        let value = Value::Object(hashmap! {
            "kind".into() => "block".into(),
            "ident".into() => Value::Array(vec!["foo".into()]),
        });

        assert!(Structure::try_from(value).is_err());

        let value = Value::Object(hashmap! {
            "kind".into() => "block".into(),
            "ident".into() => Value::Array(vec!["foo".into()]),
            "body".into() => Value::Array(vec![Value::Null])
        });

        assert!(Structure::try_from(value).is_err());

        let value = Value::Array(Vec::new());

        assert!(Structure::try_from(value).is_err());
    }
}
