// Heredoc cases from the original specsuite at https://github.com/hashicorp/hcl/blob/65731f3310963019707cb1ee851b7c12779a4f62/specsuite/tests/expressions/heredoc.hcl
result = {
  normal = {
    basic                     = "Foo\nBar\nBaz\n"
    indented                  = "    Foo\n    Bar\n    Baz\n"
    indented_more             = "    Foo\n      Bar\n    Baz\n"
    interp                    = "    Foo\n    Bar\n    Baz\n"
    newlines_between          = "Foo\n\nBar\n\nBaz\n"
    indented_newlines_between = "    Foo\n\n    Bar\n\n    Baz\n"
    marker_at_suffix          = "    NOT EOT\n"
  }
  indent = {
    basic                     = "Foo\nBar\nBaz\n"
    indented                  = "Foo\nBar\nBaz\n"
    indented_more             = "Foo\n  Bar\nBaz\n"
    indented_less             = "  Foo\nBar\n  Baz\n"
    interp                    = "Foo\nBar\nBaz\n"
    interp_indented_more      = "Foo\n  Bar\nBaz\n"
    interp_indented_less      = "  Foo\n  Bar\n  Baz\n"
    tabs                      = "Foo\n Bar\n Baz\n"
    unicode_spaces            = " Foo (there's two \"em spaces\" before Foo there)\nBar\nBaz\n"
    newlines_between          = "Foo\n\nBar\n\nBaz\n"
    indented_newlines_between = "Foo\n\nBar\n\nBaz\n"
  }
}
