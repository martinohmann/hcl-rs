equality "==" {
  exactly = "a" == "a"
  not     = "a" == "b"

  type_mismatch_number = "1" == 1
  type_mismatch_bool   = "true" == true
}

equality "!=" {
  exactly = "a" != "a"
  not     = "a" != "b"

  type_mismatch_number = "1" != 1
  type_mismatch_bool   = "true" != true
}

inequality "<" {
  lt = 1 < 2
  gt = 2 < 1
  eq = 1 < 1
}

inequality "<=" {
  lt = 1 <= 2
  gt = 2 <= 1
  eq = 1 <= 1
}

inequality ">" {
  lt = 1 > 2
  gt = 2 > 1
  eq = 1 > 1
}

inequality ">=" {
  lt = 1 >= 2
  gt = 2 >= 1
  eq = 1 >= 1
}

arithmetic {
  add      = 2 + 3.5
  // @FIXME(mohmann): This currently has a precision problem which needs to be
  // fixed. When parsed and serialized again it looks like this:
  //
  // 3.141592653589793 + 1
  // add_big  = 3.14159265358979323846264338327950288419716939937510582097494459 + 1
  sub      = 3.5 - 2
  sub_neg  = 2 - 3.5
  mul      = 2 * 4.5
  div      = 1 / 10
  mod      = 11 % 5
  mod_frac = 11 % 5.1
}

logical_binary "&&" {
  tt = true && true
  ft = false && true
  tf = true && false
  ff = false && false
}

logical_binary "||" {
  tt = true || true
  ft = false || true
  tf = true || false
  ff = false || false
}

logical_unary "!" {
  t = !true
  f = !false
}

conditional {
  t = true ? "a" : "b"
  f = false ? "a" : "b"
}
