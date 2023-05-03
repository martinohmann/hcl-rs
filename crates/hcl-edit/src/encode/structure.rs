use super::{
    encode_decorated, encode_quoted_string, Encode, EncodeDecorated, EncodeState, BOTH_SPACE_DECOR,
    LEADING_SPACE_DECOR, NO_DECOR, TRAILING_SPACE_DECOR,
};
use crate::structure::{Attribute, Block, BlockLabel, Body, Structure};
use std::fmt::{self, Write};

impl Encode for Body {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        for structure in self.iter() {
            structure.encode_decorated(buf, NO_DECOR)?;
            buf.write_char('\n')?;
        }

        Ok(())
    }
}

impl EncodeDecorated for Structure {
    fn encode_decorated(&self, buf: &mut EncodeState, default_decor: (&str, &str)) -> fmt::Result {
        match self {
            Structure::Attribute(attr) => attr.encode_decorated(buf, default_decor),
            Structure::Block(block) => block.encode_decorated(buf, default_decor),
        }
    }
}

impl Encode for Attribute {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.key.encode_decorated(buf, TRAILING_SPACE_DECOR)?;
        buf.write_char('=')?;
        self.value.encode_decorated(buf, LEADING_SPACE_DECOR)
    }
}

impl Encode for Block {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.ident.encode_decorated(buf, TRAILING_SPACE_DECOR)?;

        for label in &self.labels {
            label.encode_decorated(buf, TRAILING_SPACE_DECOR)?;
        }

        buf.write_char('{')?;

        let body = &self.body;

        encode_decorated(body, buf, NO_DECOR, |buf| {
            if body.prefer_oneline() && (body.is_empty() || body.has_single_attribute()) {
                if let Some(attr) = body.get(0).and_then(Structure::as_attribute) {
                    attr.encode_decorated(buf, BOTH_SPACE_DECOR)?;
                }
            } else {
                buf.write_char('\n')?;
                body.encode(buf)?;
            }

            Ok(())
        })?;

        buf.write_char('}')
    }
}

impl EncodeDecorated for BlockLabel {
    fn encode_decorated(&self, buf: &mut EncodeState, default_decor: (&str, &str)) -> fmt::Result {
        match self {
            BlockLabel::String(string) => encode_decorated(string, buf, default_decor, |buf| {
                encode_quoted_string(buf, string)
            }),
            BlockLabel::Ident(ident) => ident.encode_decorated(buf, default_decor),
        }
    }
}
