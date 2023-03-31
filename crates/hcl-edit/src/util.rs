use std::borrow::Cow;

pub(crate) fn dedent<'a, S>(s: S, skip_first: bool) -> Cow<'a, str>
where
    S: Into<Cow<'a, str>>,
{
    let s = s.into();
    let n = min_leading_whitespace(&s, skip_first);
    dedent_by(s, n, skip_first)
}

pub(crate) fn dedent_by<'a, S>(s: S, n: usize, skip_first: bool) -> Cow<'a, str>
where
    S: Into<Cow<'a, str>>,
{
    let s = s.into();

    if s.is_empty() || n == 0 {
        return s;
    }

    let mut dedented = String::with_capacity(s.len());

    for (i, line) in s.lines().enumerate() {
        if i == 0 && skip_first {
            dedented.push_str(line);
        } else if !line.is_empty() {
            dedented.extend(line.chars().skip(n));
        }

        dedented.push('\n');
    }

    if dedented.ends_with('\n') && !s.ends_with('\n') {
        let new_len = dedented.len() - 1;
        dedented.truncate(new_len);
    }

    dedented.shrink_to_fit();

    Cow::Owned(dedented)
}

pub(crate) fn min_leading_whitespace(s: &str, skip_first: bool) -> usize {
    if s.is_empty() {
        return 0;
    }

    let mut leading_ws: Option<usize> = None;

    // Find the minimum number of possible leading units of whitespace that can be be stripped off
    // of each non-empty line.
    for (i, line) in s.lines().enumerate() {
        if (i == 0 && skip_first) || line.is_empty() {
            continue;
        }

        let line_leading_ws = line.chars().take_while(|ch| ch.is_whitespace()).count();

        if line_leading_ws == 0 {
            // Fast path: no dedent needed if we encounter a non-empty line which starts with a
            // non-whitespace character.
            return 0;
        }

        leading_ws = Some(leading_ws.map_or(line_leading_ws, |leading_ws| {
            leading_ws.min(line_leading_ws)
        }));
    }

    leading_ws.unwrap_or(0)
}

pub(crate) fn indent_by(s: &str, n: usize, skip_first: bool) -> String {
    let prefix = " ".repeat(n);
    let length = s.len();
    let mut output = String::with_capacity(length + length / 2);

    for (i, line) in s.lines().enumerate() {
        if i > 0 {
            output.push('\n');

            if !line.is_empty() {
                output.push_str(&prefix);
            }
        } else if !skip_first && !line.is_empty() {
            output.push_str(&prefix);
        }

        output.push_str(line);
    }

    if s.ends_with('\n') {
        output.push('\n');
    }

    output
}

pub(crate) fn indent_with<'a, S>(s: S, prefix: &str, skip_first: bool) -> Cow<'a, str>
where
    S: Into<Cow<'a, str>>,
{
    let s = s.into();

    if prefix.is_empty() {
        return s;
    }

    let length = s.len();
    let mut output = String::with_capacity(length + length / 2);

    for (i, line) in s.lines().enumerate() {
        if i > 0 {
            output.push('\n');

            if !line.is_empty() {
                output.push_str(prefix);
            }
        } else if !skip_first && !line.is_empty() {
            output.push_str(prefix);
        }

        output.push_str(line);
    }

    if s.ends_with('\n') {
        output.push('\n');
    }

    Cow::Owned(output)
}
