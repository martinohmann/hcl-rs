normal {
  basic = <<EOT
Foo
Bar
Baz
EOT

  indented = <<EOT
    Foo
    Bar
    Baz
  EOT

  indented_more = <<EOT
    Foo
      Bar
    Baz
  EOT

  interp = <<EOT
    Foo
    ${bar}
    Baz
  EOT

  newlines_between = <<EOT
Foo

Bar

Baz
EOT

  indented_newlines_between = <<EOT
    Foo

    Bar

    Baz
  EOT

  marker_at_suffix = <<EOT
    NOT EOT
  EOT
}

indent {
  basic = <<-EOT
Foo
Bar
Baz
EOT

  indented = <<-EOT
    Foo
    Bar
    Baz
  EOT

  indented_more = <<-EOT
    Foo
      Bar
    Baz
  EOT

  indented_less = <<-EOT
    Foo
  Bar
    Baz
  EOT

  interp = <<-EOT
    Foo
    ${bar}
    Baz
  EOT

  interp_indented_more = <<-EOT
    Foo
      ${bar}
    Baz
  EOT

  interp_indented_less = <<-EOT
    Foo
  ${space_bar}
    Baz
  EOT

  newlines_between = <<-EOT
Foo

Bar

Baz
EOT

  indented_newlines_between = <<-EOT
    Foo

    Bar

    Baz
  EOT
}
