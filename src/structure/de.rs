use super::{Body, Structure};
use crate::value::Map;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

impl<'de> Deserialize<'de> for Body {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BodyVisitor;

        impl<'de> Visitor<'de> for BodyVisitor {
            type Value = Body;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a HCL config file or block body")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let mut vec = Vec::with_capacity(visitor.size_hint().unwrap_or(0));

                while let Some(structure) = visitor.next_element()? {
                    vec.push(structure);
                }

                Ok(Body::from_iter(vec))
            }
        }

        deserializer.deserialize_any(BodyVisitor)
    }
}

impl<'de> Deserialize<'de> for Structure {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StructureVisitor;

        impl<'de> Visitor<'de> for StructureVisitor {
            type Value = Structure;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a HCL structure")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut map = Map::with_capacity(visitor.size_hint().unwrap_or(0));

                while let Some((key, value)) = visitor.next_entry()? {
                    map.insert(key, value);
                }

                Structure::try_from(map).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(StructureVisitor)
    }
}

#[cfg(test)]
mod test {
    use crate::de::from_str;
    use crate::structure::{Attribute, Block, Body};
    use crate::value::Value;

    #[test]
    fn deserialize_structure() {
        let hcl = r#"
            foo = 42

            block {
              bar = true
              baz = [var.enabled, 1, "two"]
            }
        "#;
        let body: Body = from_str(hcl).unwrap();
        let expected = Body::from_iter(vec![
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

        assert_eq!(body, expected);
    }
}
