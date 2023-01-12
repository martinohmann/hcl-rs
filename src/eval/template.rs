use super::*;

pub(super) fn evaluate_template(
    result: &mut String,
    template: &Template,
    ctx: &Context,
    strip: StripMode,
) -> EvalResult<()> {
    let elems = template.elements();
    let len = elems.len();

    for (index, elem) in elems.iter().enumerate() {
        let strip = if index == 0 && len == 1 {
            strip
        } else if index == 0 {
            StripMode::from((strip.strip_start(), elems[index + 1].strip_start()))
        } else if index == len - 1 {
            StripMode::from((elems[index - 1].strip_end(), strip.strip_end()))
        } else {
            StripMode::from((elems[index - 1].strip_end(), elems[index + 1].strip_start()))
        };

        evaluate_element(result, elem, ctx, strip)?;
    }

    Ok(())
}

fn evaluate_element(
    result: &mut String,
    element: &Element,
    ctx: &Context,
    strip: StripMode,
) -> EvalResult<()> {
    match element {
        Element::Literal(literal) => {
            result.push_str(strip_literal(literal, strip));
            Ok(())
        }
        Element::Interpolation(interp) => evaluate_interpolation(result, interp, ctx),
        Element::Directive(dir) => evaluate_directive(result, dir, ctx),
    }
}

// Depending on the `StripMode`, strips off leading and trailing spaces up until the first line
// break that is encountered. The line break is stripped as well.
fn strip_literal(literal: &str, strip: StripMode) -> &str {
    fn is_space(ch: char) -> bool {
        ch.is_whitespace() && ch != '\r' && ch != '\n'
    }

    fn trim_start(s: &str) -> &str {
        let s = s.trim_start_matches(is_space);

        s.strip_prefix("\r\n")
            .or_else(|| s.strip_prefix('\n'))
            .unwrap_or(s)
    }

    fn trim_end(s: &str) -> &str {
        let s = s.trim_end_matches(is_space);

        s.strip_suffix("\r\n")
            .or_else(|| s.strip_suffix('\n'))
            .unwrap_or(s)
    }

    match strip {
        StripMode::Both => trim_start(trim_end(literal)),
        StripMode::Start => trim_start(literal),
        StripMode::End => trim_end(literal),
        StripMode::None => literal,
    }
}

fn evaluate_interpolation(
    result: &mut String,
    interp: &Interpolation,
    ctx: &Context,
) -> EvalResult<()> {
    let string = match interp.expr.evaluate(ctx)? {
        Value::String(string) => string,
        other => other.to_string(),
    };

    result.push_str(&string);
    Ok(())
}

fn evaluate_directive(result: &mut String, dir: &Directive, ctx: &Context) -> EvalResult<()> {
    match dir {
        Directive::If(dir) => evaluate_if_directive(result, dir, ctx),
        Directive::For(dir) => evaluate_for_directive(result, dir, ctx),
    }
}

fn evaluate_if_directive(result: &mut String, dir: &IfDirective, ctx: &Context) -> EvalResult<()> {
    if expr::evaluate_bool(&dir.cond_expr, ctx)? {
        let next_strip = if dir.false_template.is_some() {
            dir.else_strip
        } else {
            dir.endif_strip
        };
        let strip = StripMode::from_adjacent(dir.if_strip, next_strip);
        evaluate_template(result, &dir.true_template, ctx, strip)?;
    } else if let Some(false_template) = &dir.false_template {
        let strip = StripMode::from_adjacent(dir.else_strip, dir.endif_strip);
        evaluate_template(result, false_template, ctx, strip)?;
    }

    Ok(())
}

fn evaluate_for_directive(
    result: &mut String,
    dir: &ForDirective,
    ctx: &Context,
) -> EvalResult<()> {
    let strip = StripMode::from_adjacent(dir.for_strip, dir.endfor_strip);
    let collection = expr::Collection::from_for_directive(dir, ctx)?;

    for ctx in collection {
        evaluate_template(result, &dir.template, &ctx?, strip)?;
    }

    Ok(())
}
