# Specsuite Documentation

The test file `<test-name>.hcl` is any valid HCL file. The expected test
results are stored in `<test-name>.t`. Additionally, the optional file
`<test-name>.hcldec` may contain variable definitions that are used to evaluate
HCL expressions.

**Note**: The specsuite is modelled after the [original
specsuite](https://github.com/hashicorp/hcl/tree/main/specsuite) but not all
test cases are implemented yet.

## `.hcldec` file structure

| Field       | Type   | Description |
|-------------|--------|-------------|
| `ignore`    | bool   | If `false` the test result must match the parsed HCL, or parser's error message.  If `true`, it must not match<sup>1</sup>. |
| `message`   | String | Test comment, output on test failure. |
| `variables` | Object | Variable values used to evaluate HCL expressions and templates. |

### Example

```hcl
ignore = false

variables {
  foo = "bar"
}
```

## `.t` file structure

| Field         | Type   | Description |
|---------------|--------|-------------|
| `result`      | Object | If the test expects the input HCL to parse and evaluate correctly, this contains the expected result. |
| `diagnostics` | Object | If an error is expected, the `error` attribute in this block contains the expected error message. |

To bootstrap tests, if `<test-name>.t` does not exist the test will be ignored
and it will dump the parsed HCL result, which can be verified and turned into a
suitable `.t` file.

<sup>1</sup> See truth table, but if the test passes and `ignore` is set to
`true`, it's treated as a failure and will tell you to fix your test.

### Example

```hcl
result = {
  foo = "bar"
}
```

```hcl
diagnostics {
  error = "Some parser error"
}
```

## Test Truth Table

| Check                   | ignore   | Status |
|-------------------------|----------|--------|
| `result == expectation` | `false`  |  pass  |
| `result != expectation` | `true`   |  pass  |
| `result == expectation` | `true`   |  fail  |
| `result != expectation` | `false`  |  fail  |
