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
            StripMode::from((strip.strip_start(), elems[index + 1].strip_prev_end()))
        } else if index == len - 1 {
            StripMode::from((elems[index - 1].strip_next_start(), strip.strip_end()))
        } else {
            StripMode::from((
                elems[index - 1].strip_next_start(),
                elems[index + 1].strip_prev_end(),
            ))
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
            let stripped = match strip {
                StripMode::Both => literal.trim(),
                StripMode::Start => literal.trim_start(),
                StripMode::End => literal.trim_end(),
                StripMode::None => literal,
            };
            result.push_str(stripped);
            Ok(())
        }
        Element::Interpolation(interp) => evaluate_interpolation(result, interp, ctx),
        Element::Directive(dir) => evaluate_directive(result, dir, ctx),
    }
}

fn evaluate_interpolation(
    result: &mut String,
    interp: &Interpolation,
    ctx: &Context,
) -> EvalResult<()> {
    let string = match interp.expr.evaluate(ctx)? {
        Value::String(string) => string,
        other => crate::format::to_string(&other)?,
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
    let collection = Collection::from_for_directive(dir, ctx)?;
    let len = collection.len();

    for (index, ctx) in collection.into_iter().enumerate() {
        let ctx = &ctx?;

        let strip = if index == 0 && len == 1 {
            StripMode::from_adjacent(dir.for_strip, dir.endfor_strip)
        } else if index == 0 {
            StripMode::from_adjacent(dir.for_strip, StripMode::None)
        } else if index == len - 1 {
            StripMode::from_adjacent(StripMode::None, dir.endfor_strip)
        } else {
            StripMode::None
        };

        evaluate_template(result, &dir.template, ctx, strip)?;
    }

    Ok(())
}
