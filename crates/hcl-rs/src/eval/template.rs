use super::*;

pub(super) fn evaluate_template(
    result: &mut String,
    template: &Template,
    ctx: &Context,
    prev_strip: Strip,
    next_strip: Strip,
) -> EvalResult<()> {
    let elems = template.elements();
    let len = elems.len();

    for (index, elem) in elems.iter().enumerate() {
        let (prev, next) = if index == 0 && len == 1 {
            (prev_strip, next_strip)
        } else if index == 0 {
            (prev_strip, elems[index + 1].strip())
        } else if index == len - 1 {
            (elems[index - 1].strip(), next_strip)
        } else {
            (elems[index - 1].strip(), elems[index + 1].strip())
        };

        evaluate_element(result, elem, ctx, prev, next)?;
    }

    Ok(())
}

fn evaluate_element(
    result: &mut String,
    element: &Element,
    ctx: &Context,
    prev_strip: Strip,
    next_strip: Strip,
) -> EvalResult<()> {
    match element {
        Element::Literal(literal) => {
            result.push_str(strip_literal(literal, prev_strip, next_strip));
            Ok(())
        }
        Element::Interpolation(interp) => evaluate_interpolation(result, interp, ctx),
        Element::Directive(dir) => evaluate_directive(result, dir, ctx),
    }
}

// Depending on the `StripMode`, strips off leading and trailing spaces up until the first line
// break that is encountered. The line break is stripped as well.
fn strip_literal(mut literal: &str, prev_strip: Strip, next_strip: Strip) -> &str {
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

    if prev_strip.strip_end() {
        literal = trim_start(literal);
    }

    if next_strip.strip_start() {
        literal = trim_end(literal);
    }

    literal
}

fn evaluate_interpolation(
    result: &mut String,
    interp: &Interpolation,
    ctx: &Context,
) -> EvalResult<()> {
    let string =
        match interp.expr.evaluate(ctx)? {
            Value::String(string) => string,
            Value::Capsule(_) => return Err(Error::new(
                "cannot format capsule value returned while evaluating an interpolation as string",
            )),
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
        evaluate_template(result, &dir.true_template, ctx, dir.if_strip, next_strip)?;
    } else if let Some(false_template) = &dir.false_template {
        evaluate_template(result, false_template, ctx, dir.else_strip, dir.endif_strip)?;
    }

    Ok(())
}

fn evaluate_for_directive(
    result: &mut String,
    dir: &ForDirective,
    ctx: &Context,
) -> EvalResult<()> {
    let collection = expr::Collection::from_for_directive(dir, ctx)?;

    for ctx in collection {
        evaluate_template(
            result,
            &dir.template,
            &ctx?,
            dir.for_strip,
            dir.endfor_strip,
        )?;
    }

    Ok(())
}
