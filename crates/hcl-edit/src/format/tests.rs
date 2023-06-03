use super::*;
use crate::structure::Body;
use pretty_assertions::assert_eq;

#[test]
fn default_format_body() {
    let input = r#"
    // comment
block  "label"  {  # comment
    // comment
attr1 = "value"
    attr2 = 42

// another comment
nested_block {
foo = 1  # foo comment

    object = { foo :bar, baz= qux,  }

    multiline_object = { foo = bar/*comment */,
     /* comment */baz = qux, one =/*comment*/1, multi = 42 /*
  multiline comment */
    // another
      # and another
two:2 }
}

    array = [1,     /* two */ 2, 3 ,      ]

      multiline_array    =    [

      1
      /* comment */
    ,
    2,
        3 /* comment */,
  /* comment*/

  4

  ,
        ]

    bar =   func(1, [
        2, 3])

    baz  = func(
     1, [
        2, /* three */ 3])

qux = func( 1  , /*two*/3  ...  )
  }

  /* some trailing comment */"#;

    let expected = r#"
// comment
block "label" { # comment
  // comment
  attr1 = "value"
  attr2 = 42

  // another comment
  nested_block {
    foo = 1 # foo comment

    object = { foo = bar, baz = qux, }

    multiline_object = {
      foo = bar /*comment */
      /* comment */ baz = qux
      one = /*comment*/ 1
      multi = 42 /*
  multiline comment */
      // another
      # and another
      two = 2
    }
  }

  array = [1, /* two */ 2, 3, ]

  multiline_array = [

    1
    /* comment */
    ,
    2,
    3 /* comment */,
    /* comment*/

    4

    ,
  ]

  bar = func(1, [
    2,
    3
  ])

  baz = func(
    1,
    [
      2,
      /* three */ 3
    ]
  )

  qux = func(1, /*two*/ 3...)
}

/* some trailing comment */"#;

    let mut body = input.parse::<Body>().unwrap();
    body.default_format();

    assert_eq!(body.to_string(), expected);
}
