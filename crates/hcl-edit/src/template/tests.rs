use super::*;
use pretty_assertions::assert_eq;

#[test]
fn dedent_template() {
    let tests = [
        ("${foo}\n  bar\n", "${foo}\n  bar\n"),
        ("  ${foo}\n  ${bar}\n    ${baz}", "${foo}\n${bar}\n  ${baz}"),
        ("  ${foo}\n", "${foo}\n"),
        ("  foo\n${bar}\n    baz\n", "  foo\n${bar}\n    baz\n"),
        ("  foo${bar}\n    baz", "foo${bar}\n  baz"),
        ("  foo\n    bar\n      baz", "foo\n  bar\n    baz"),
    ];

    for (input, expected) in tests {
        let mut template: Template = input.parse().unwrap();
        template.dedent();

        assert_eq!(
            template.to_string(),
            expected,
            "unexpected dedent result for input `{input}`",
        );
    }
}
