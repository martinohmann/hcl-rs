use super::{
    encode_escaped, Encode, EncodeDecorated, EncodeState, BOTH_SPACE_DECOR, LEADING_SPACE_DECOR,
    TRAILING_SPACE_DECOR,
};
use crate::template::{
    Directive, Element, ElseTemplateExpr, EndforTemplateExpr, EndifTemplateExpr, EscapedLiteral,
    ForDirective, ForTemplateExpr, HeredocTemplate, IfDirective, IfTemplateExpr, Interpolation,
    StringTemplate, Strip, Template,
};
use crate::util::indent_by;
use std::fmt::{self, Write};

const INTERPOLATION_START: &str = "${";
const DIRECTIVE_START: &str = "%{";
const ESCAPED_INTERPOLATION: &str = "$${";
const ESCAPED_DIRECTIVE: &str = "%%{";

impl Encode for StringTemplate {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_char('"')?;
        buf.escaped(|buf| {
            for element in self.iter() {
                element.encode(buf)?;
            }

            Ok(())
        })?;
        buf.write_char('"')
    }
}

impl Encode for HeredocTemplate {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_str("<<")?;

        if self.indent().is_some() {
            buf.write_char('-')?;
        }

        writeln!(buf, "{}", self.delimiter.as_str())?;

        match self.indent() {
            Some(n) => {
                let mut indent_buf = String::new();
                let mut indent_state = EncodeState::new(&mut indent_buf);
                self.template.encode(&mut indent_state)?;
                let indented = indent_by(&indent_buf, n, false);
                buf.write_str(&indented)?;
            }
            None => self.template.encode(buf)?,
        }

        self.trailing().encode_with_default(buf, "")?;

        write!(buf, "{}", self.delimiter.as_str())
    }
}

impl Encode for Template {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        for element in self.iter() {
            element.encode(buf)?;
        }

        Ok(())
    }
}

impl Encode for Element {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        match self {
            Element::Literal(lit) => {
                if buf.escape() {
                    encode_escaped(buf, lit)
                } else {
                    buf.write_str(lit)
                }
            }
            Element::EscapedLiteral(lit) => buf.write_str(match lit.value() {
                EscapedLiteral::Interpolation => ESCAPED_INTERPOLATION,
                EscapedLiteral::Directive => ESCAPED_DIRECTIVE,
            }),
            Element::Interpolation(interp) => interp.encode(buf),
            Element::Directive(dir) => dir.encode(buf),
        }
    }
}

impl Encode for Interpolation {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        encode_strip(buf, INTERPOLATION_START, self.strip, |buf| {
            self.expr.encode_decorated(buf, BOTH_SPACE_DECOR)
        })
    }
}

impl Encode for Directive {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        match self {
            Directive::If(dir) => dir.encode(buf),
            Directive::For(dir) => dir.encode(buf),
        }
    }
}

impl Encode for IfDirective {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.if_expr.encode(buf)?;
        if let Some(else_expr) = &self.else_expr {
            else_expr.encode(buf)?;
        }
        self.endif_expr.encode(buf)
    }
}

impl Encode for IfTemplateExpr {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        encode_strip(buf, DIRECTIVE_START, self.strip, |buf| {
            self.preamble().encode_with_default(buf, " ")?;
            buf.write_str("if")?;
            self.cond_expr.encode_decorated(buf, TRAILING_SPACE_DECOR)
        })?;
        self.template.encode(buf)
    }
}

impl Encode for ElseTemplateExpr {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        encode_strip(buf, DIRECTIVE_START, self.strip, |buf| {
            self.preamble().encode_with_default(buf, " ")?;
            buf.write_str("else")?;
            self.trailing().encode_with_default(buf, " ")
        })?;
        self.template.encode(buf)
    }
}

impl Encode for EndifTemplateExpr {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        encode_strip(buf, DIRECTIVE_START, self.strip, |buf| {
            self.preamble().encode_with_default(buf, " ")?;
            buf.write_str("endif")?;
            self.trailing().encode_with_default(buf, " ")
        })
    }
}

impl Encode for ForDirective {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.for_expr.encode(buf)?;
        self.endfor_expr.encode(buf)
    }
}

impl Encode for ForTemplateExpr {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        encode_strip(buf, DIRECTIVE_START, self.strip, |buf| {
            self.preamble().encode_with_default(buf, " ")?;
            buf.write_str("for")?;

            if let Some(key_var) = &self.key_var {
                key_var.encode_decorated(buf, LEADING_SPACE_DECOR)?;
                buf.write_char(',')?;
            }

            self.value_var.encode_decorated(buf, BOTH_SPACE_DECOR)?;
            buf.write_str("in")?;
            self.collection_expr.encode_decorated(buf, BOTH_SPACE_DECOR)
        })?;
        self.template.encode(buf)
    }
}

impl Encode for EndforTemplateExpr {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        encode_strip(buf, DIRECTIVE_START, self.strip, |buf| {
            self.preamble().encode_with_default(buf, " ")?;
            buf.write_str("endfor")?;
            self.trailing().encode_with_default(buf, " ")
        })
    }
}

fn encode_strip<F>(buf: &mut EncodeState, start_marker: &str, strip: Strip, f: F) -> fmt::Result
where
    F: FnOnce(&mut EncodeState) -> fmt::Result,
{
    buf.write_str(start_marker)?;
    if strip.strip_start() {
        buf.write_char('~')?;
    }

    f(buf)?;

    if strip.strip_end() {
        buf.write_char('~')?;
    }

    buf.write_char('}')
}
