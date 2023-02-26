use std::fmt;

pub(crate) fn write_escaped(buf: &mut dyn fmt::Write, value: &str) -> fmt::Result {
    for ch in value.chars() {
        match ch {
            '\u{8}' => buf.write_str("\\b")?,
            '\u{9}' => buf.write_str("\\t")?,
            '\u{a}' => buf.write_str("\\n")?,
            '\u{c}' => buf.write_str("\\f")?,
            '\u{d}' => buf.write_str("\\r")?,
            '\u{22}' => buf.write_str("\\\"")?,
            '\u{5c}' => buf.write_str("\\\\")?,
            c if c <= '\u{1f}' || c == '\u{7f}' => {
                write!(buf, "\\u{:04X}", ch as u32)?;
            }
            ch => buf.write_char(ch)?,
        }
    }

    Ok(())
}
