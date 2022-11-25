use crate::expr::{Expression, Variable};
use crate::structure::{Attribute, Block, Body, Structure};
use crate::Identifier;
use indexmap::{indexmap, IndexMap};
use pretty_assertions::assert_eq;
use serde::{ser, Serialize};

#[track_caller]
fn assert_body<T>(given: T, expected: Body)
where
    T: ser::Serialize,
{
    assert_eq!(Body::from_serializable(&given).unwrap(), expected);
}

#[test]
fn roundtrip() {
    assert_body(Body::default(), Body::default());
    assert_body(
        Body::builder()
            .add_attribute(("foo", "bar"))
            .add_block(Block::builder("baz").build())
            .build(),
        Body::builder()
            .add_attribute(("foo", "bar"))
            .add_block(Block::builder("baz").build())
            .build(),
    );
    assert_body(
        Structure::Block(
            Block::builder("foo")
                .add_label("bar")
                .add_label(Identifier::unchecked("bar"))
                .add_attribute(("baz", "qux"))
                .build(),
        ),
        Block::builder("foo")
            .add_label("bar")
            .add_label(Identifier::unchecked("bar"))
            .add_attribute(("baz", "qux"))
            .build()
            .into(),
    );
}

#[test]
fn builtin() {
    assert_body(
        Attribute::new("foo", "bar"),
        Attribute::new("foo", "bar").into(),
    );
    assert_body(
        Attribute::new("foo", vec!["bar", "baz"]),
        Attribute::new("foo", vec!["bar", "baz"]).into(),
    );
    assert_body(
        Block::builder("foo")
            .add_label("bar")
            .add_label(Identifier::unchecked("bar"))
            .add_attribute(("baz", "qux"))
            .build(),
        Block::builder("foo")
            .add_label("bar")
            .add_label(Identifier::unchecked("bar"))
            .add_attribute(("baz", "qux"))
            .build()
            .into(),
    );
}

#[test]
fn custom_blocks() {
    #[derive(Serialize)]
    #[serde(rename = "$hcl::Block")]
    struct Unlabeled<T>(T);

    #[derive(Serialize)]
    #[serde(rename = "$hcl::LabeledBlock")]
    struct Labeled<T>(T);

    #[derive(Serialize)]
    struct A {
        a: u8,
    }

    #[derive(Serialize)]
    struct B {
        b: u8,
    }

    #[derive(Serialize)]
    struct C<A, B> {
        a: A,
        b: B,
    }

    #[derive(Serialize)]
    struct D<T> {
        d: T,
    }

    #[derive(Serialize)]
    struct Config {
        attr: Expression,
        unlabeled: Unlabeled<A>,
        nested: Labeled<C<Labeled<C<Unlabeled<A>, Unlabeled<D<Expression>>>>, Unlabeled<B>>>,
        nested_many: Labeled<D<Labeled<Vec<Unlabeled<A>>>>>,
        map: Labeled<IndexMap<&'static str, Labeled<IndexMap<&'static str, Unlabeled<A>>>>>,
    }

    let given = Config {
        attr: Expression::Variable(Variable::unchecked("var")),
        unlabeled: Unlabeled(A { a: 1 }),
        nested: Labeled(C {
            a: Labeled(C {
                a: Unlabeled(A { a: 2 }),
                b: Unlabeled(D {
                    d: Expression::Null,
                }),
            }),
            b: Unlabeled(B { b: 2 }),
        }),
        nested_many: Labeled(D {
            d: Labeled(vec![Unlabeled(A { a: 3 }), Unlabeled(A { a: 4 })]),
        }),
        map: Labeled(indexmap! {
            "a" => Labeled(indexmap! {
                "a" => Unlabeled(A { a: 5 }),
                "b" => Unlabeled(A { a: 6 }),
            }),
            "b" => Labeled(indexmap! {
                "a" => Unlabeled(A { a: 7 }),
                "b" => Unlabeled(A { a: 8 }),
            }),
        }),
    };

    let expected = Body::builder()
        .add_attribute(Attribute::new("attr", Variable::unchecked("var")))
        .add_block(Block::builder("unlabeled").add_attribute(("a", 1)).build())
        .add_block(
            Block::builder("nested")
                .add_labels(["a", "a"])
                .add_attribute(("a", 2))
                .build(),
        )
        .add_block(
            Block::builder("nested")
                .add_labels(["a", "b"])
                .add_attribute(("d", Expression::Null))
                .build(),
        )
        .add_block(
            Block::builder("nested")
                .add_label("b")
                .add_attribute(("b", 2))
                .build(),
        )
        .add_block(
            Block::builder("nested_many")
                .add_label("d")
                .add_attribute(("a", 3))
                .build(),
        )
        .add_block(
            Block::builder("nested_many")
                .add_label("d")
                .add_attribute(("a", 4))
                .build(),
        )
        .add_block(
            Block::builder("map")
                .add_labels(["a", "a"])
                .add_attribute(("a", 5))
                .build(),
        )
        .add_block(
            Block::builder("map")
                .add_labels(["a", "b"])
                .add_attribute(("a", 6))
                .build(),
        )
        .add_block(
            Block::builder("map")
                .add_labels(["b", "a"])
                .add_attribute(("a", 7))
                .build(),
        )
        .add_block(
            Block::builder("map")
                .add_labels(["b", "b"])
                .add_attribute(("a", 8))
                .build(),
        )
        .build();

    assert_body(given, expected);
}
