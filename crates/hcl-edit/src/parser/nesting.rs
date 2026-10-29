//! Nesting depth tracking which prevents the recursive parser from overflowing the stack on deeply
//! nested input.

use super::prelude::*;

use winnow::combinator::{cut_err, fail};

/// The maximum nesting depth of expressions, templates and blocks.
///
/// Root expressions like attribute values do not count towards the nesting depth, only the
/// expressions, templates and blocks nested within them do.
///
/// Input exceeding this limit is rejected with a parse error instead of overflowing the stack.
pub(super) const MAX_NESTING_DEPTH: usize = 64;

/// The maximum number of binary operations on any path through an expression tree.
///
/// Chained binary operators are parsed iteratively, but produce a left-deep expression tree with
/// one level per operator which is traversed recursively after parsing (e.g. when converting,
/// formatting or dropping it). Since operands may contain binary operations themselves, the
/// operators of nested chains add up.
///
/// Input exceeding this limit is rejected with a parse error instead of producing an expression
/// tree that overflows the stack when traversed.
pub(super) const MAX_OPERATOR_DEPTH: usize = 128;

/// Nesting state of the parser, tracked as part of the parser input state.
#[derive(Debug, Default, Clone, Copy)]
pub(super) struct NestingState {
    /// The number of nested levels the parser is currently in.
    depth: usize,
    /// The maximum operator depth of the expressions parsed within the current operator depth
    /// scope.
    operator_depth: usize,
}

/// Runs `parser` one nesting level deeper within its own operator depth scope (see
/// [`operator_scope`]).
///
/// The nesting depth is restored afterwards, regardless of whether parsing succeeded or not.
///
/// # Errors
///
/// Returns a cut error if the maximum nesting depth would be exceeded.
pub(super) fn nested<'i, P, O>(parser: P) -> impl ModalParser<Input<'i>, O, ContextError>
where
    P: ModalParser<Input<'i>, O, ContextError>,
{
    let mut parser = operator_scope(parser);

    move |input: &mut Input<'i>| {
        let depth = input.state.depth;

        if depth >= MAX_NESTING_DEPTH {
            // Keep the description in sync with `MAX_NESTING_DEPTH`.
            return cut_err(fail)
                .context(StrContext::Label("nesting depth"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "at most 64 nested levels",
                )))
                .parse_next(input);
        }

        input.state.depth = depth + 1;
        let result = parser.parse_next(input);
        // The nesting state is not part of the input checkpoint and is thus not restored when the
        // parser backtracks, so it must always be restored explicitly.
        input.state.depth = depth;
        result
    }
}

/// Runs `parser` within its own operator depth scope without increasing the nesting depth.
///
/// The operator depth starts at zero within the scope and the operator depth of everything parsed
/// by `parser` is propagated to the enclosing scope afterwards, so that the operators of sibling
/// expressions do not add up.
pub(super) fn operator_scope<'i, P, O>(
    mut parser: P,
) -> impl ModalParser<Input<'i>, O, ContextError>
where
    P: ModalParser<Input<'i>, O, ContextError>,
{
    move |input: &mut Input<'i>| {
        let operator_depth = input.state.operator_depth;
        input.state.operator_depth = 0;
        let result = parser.parse_next(input);
        input.state.operator_depth = operator_depth.max(input.state.operator_depth);
        result
    }
}

/// Checks that a chain of `operators` binary operators on top of the operands parsed within the
/// current operator depth scope does not exceed the maximum operator depth.
///
/// # Errors
///
/// Returns a cut error at `operator_start` if the maximum operator depth is exceeded.
pub(super) fn check_operator_depth<'i>(
    input: &mut Input<'i>,
    operators: usize,
    operator_start: &<Input<'i> as Stream>::Checkpoint,
) -> ModalResult<()> {
    if input.state.operator_depth + operators > MAX_OPERATOR_DEPTH {
        input.reset(operator_start);

        // Keep the description in sync with `MAX_OPERATOR_DEPTH`.
        return cut_err(fail)
            .context(StrContext::Label("binary operation"))
            .context(StrContext::Expected(StrContextValue::Description(
                "at most 128 chained binary operators",
            )))
            .parse_next(input);
    }

    Ok(())
}

/// Records that a chain of `operators` binary operators was built on top of the operands parsed
/// within the current operator depth scope.
pub(super) fn add_operator_depth(input: &mut Input, operators: usize) {
    input.state.operator_depth += operators;
}
