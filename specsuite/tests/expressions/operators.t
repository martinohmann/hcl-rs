// Operator cases from the original specsuite at https://github.com/hashicorp/hcl/blob/ee38c67330bd3d45c373e4cc0166f4855d158339/specsuite/tests/expressions/operators.hcl
result = {
  equality = {
    "==" = {
      exactly              = true
      not                  = false
      type_mismatch_number = false
      type_mismatch_bool   = false
    }
    "!=" = {
      exactly              = false
      not                  = true
      type_mismatch_number = true
      type_mismatch_bool   = true
    }
  }
  inequality = {
    "<" = {
      lt = true
      gt = false
      eq = false
    }
    "<=" = {
      lt = true
      gt = false
      eq = true
    }
    ">" = {
      lt = false
      gt = true
      eq = false
    }
    ">=" = {
      lt = false
      gt = true
      eq = true
    }
  }
  arithmetic = {
    add      = 5.5
    add_big  = 4.141592653589793
    sub      = 1.5
    sub_neg  = -1.5
    mul      = 9.0
    div      = 0.1
    mod      = 1
    mod_frac = 0.8000000000000007
  }
  logical_binary = {
    "&&" = {
      tt = true
      ft = false
      tf = false
      ff = false
    }
    "||" = {
      tt = true
      ft = true
      tf = true
      ff = false
    }
  }
  logical_unary = {
    "!" = {
      t = false
      f = true
    }
  }
  conditional = {
    t = "a"
    f = "b"
  }
}
