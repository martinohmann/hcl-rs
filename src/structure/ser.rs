use super::{Attribute, Block, Body, Structure};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

impl Serialize for Body {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;

        for structure in self.iter() {
            seq.serialize_element(structure)?;
        }

        seq.end()
    }
}

impl Serialize for Structure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Attribute(attr) => attr.serialize(serializer),
            Self::Block(block) => block.serialize(serializer),
        }
    }
}

impl Serialize for Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("kind", "attribute")?;
        map.serialize_entry("key", self.key())?;
        map.serialize_entry("value", self.value())?;
        map.end()
    }
}

impl Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(4))?;
        map.serialize_entry("kind", "block")?;
        map.serialize_entry("ident", self.ident())?;
        map.serialize_entry("keys", self.keys())?;
        map.serialize_entry("body", self.body())?;
        map.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::value::Value;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn body_to_json() {
        let body = Body::from_iter(vec![
            Attribute::new("foo".into(), 42.into()).into(),
            Block::new(
                "block",
                vec![],
                vec![
                    Attribute::new("bar".into(), true.into()).into(),
                    Attribute::new(
                        "baz".into(),
                        Value::Array(vec!["${var.enabled}".into(), 1.into(), "two".into()]),
                    )
                    .into(),
                ],
            )
            .into(),
        ]);

        let value = serde_json::to_value(body).unwrap();

        let expected = json!([
             {
                 "kind": "attribute",
                 "key": "foo",
                 "value": 42,
             },
             {
                 "kind": "block",
                 "ident": "block",
                 "keys": [],
                 "body": [
                     {
                        "kind": "attribute",
                        "key": "bar",
                        "value": true,
                     },
                     {
                        "kind": "attribute",
                        "key": "baz",
                        "value": [
                            "${var.enabled}",
                            1,
                            "two"
                        ],
                     }
                 ]
             }
        ]);

        assert_eq!(value, expected);
    }
}
