//! Deeply nested inputs for testing the parser's nesting limits.

/// Inputs nested `n` levels deep for every construct that recurses in the parser.
pub fn nested_inputs(n: usize) -> Vec<(&'static str, String)> {
    vec![
        ("array", format!("x = {}1{}", "[".repeat(n), "]".repeat(n))),
        (
            "object",
            format!("x = {}1{}", "{a = ".repeat(n), "}".repeat(n)),
        ),
        (
            "parenthesis",
            format!("x = {}1{}", "(".repeat(n), ")".repeat(n)),
        ),
        ("not", format!("x = {}true", "!".repeat(n))),
        ("neg", format!("x = {}1", "-".repeat(n))),
        ("conditional", format!("x = {}1", "true ? 1 : ".repeat(n))),
        (
            "func call",
            format!("x = {}1{}", "f(".repeat(n), ")".repeat(n)),
        ),
        ("index", format!("x = {}0{}", "a[".repeat(n), "]".repeat(n))),
        (
            "interpolation",
            format!("x = {}1{}", "\"${".repeat(n), "}\"".repeat(n)),
        ),
        (
            "directive",
            format!(
                "x = \"{}x{}\"",
                "%{if true}".repeat(n),
                "%{endif}".repeat(n)
            ),
        ),
        ("block", format!("{}{}", "a {\n".repeat(n), "}\n".repeat(n))),
    ]
}

/// Inputs with an operator depth of `n`, i.e. `n` binary operations on the deepest path through the
/// expression tree.
pub fn binary_inputs(n: usize) -> Vec<(&'static str, String)> {
    let ops = |n| " + 1".repeat(n);
    let (lhs, rhs) = (n / 2, n - n / 2);

    vec![
        ("flat", format!("x = 1{}", ops(n))),
        (
            "inner",
            format!("x = {}1{}{}", "[".repeat(64), ops(n), "]".repeat(64)),
        ),
        (
            "outer",
            format!("x = {}1{}{}", "[".repeat(64), "]".repeat(64), ops(n)),
        ),
        // The operators of a chain nested in the first operand of another chain add up.
        ("split array", format!("x = [1{}]{}", ops(lhs), ops(rhs))),
        (
            "split parenthesis",
            format!("x = (1{}){}", ops(lhs), ops(rhs)),
        ),
    ]
}

/// Runs `f` on a thread with a 2 MiB stack, which is the default for spawned threads.
///
/// Pinning the stack size makes sure that tests verify the nesting limits against it, regardless
/// of the environment (e.g. `RUST_MIN_STACK` or test runners using the main thread).
pub fn with_small_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}
