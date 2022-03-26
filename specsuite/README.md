# Specsuite Documentation

The test file `<test-name>.hcl` is any valid HCL file.  The expected test results are stored in `<test-name>.hcl.json`

## JSON Format

Structure for `<test-name>.hcl.json`

```json
{
  "ignore": true,
  "message": "Auto-generated test. Verify for accuracy.",
  "body": {
    "heredoc": {
      "data": "Indent was stripped.\n  And it was the correct amount.\n"
    }
  }
}
```

### JSON Fields

| Field     | Type   | Description |
|-----------|--------|-------------|
| `ignore`  | bool   | If false the body field must match the parsed HCL, or Parser's error message.  If false, it must not match<sup>1</sup>. |
| `message` | String | Test comment, output on test failure. |
| `body`    | Object | JSON encoded output of `hcl::from_str` or the error message produced. The parsed HCL is checked against this for success. |

To bootstrap tests, if `<test-name>.hcl.json` does not exist or fails to parse the test will be ignored and it will dump the parsed HCL file as JSON.  Which can be verified and turned into the JSON file.

<sup>1</sup> See truth table but if the test passes but ignore is set to true, it's treated as a failure and will tell you to fix your test.

## Test Truth Table
|       Check        |  ignore  | Status |
|--------------------|----------|--------|
| `HCL == JSON.body` | `false`  |  pass  |
| `HCL != JSON.body` |  `true`  |  pass  |
| `HCL == JSON.body` |  `true`  |  fail  |
| `HCL != JSON.body` | `false`  |  fail  |
