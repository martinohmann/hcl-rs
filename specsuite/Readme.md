# Specsuite Documentation

The test file `<test-name>.hcl` is any valid HCL file.  The expected test results are stored in `<test-name>.hcl.json`

## JSON Format
Structure for `<test-name>.hcl.json`
```
{
  "ignore": true,
  "message": "Auto-generated test. Verify for accuracy.",
  "body": {
    "heredoc": {
      "data": "Indent was stripped.\n  And it was the correct amount."
    }
  }
}
```

### JSON Fields
| Field   |  Type  | Description |
|---------|--------|-------------|
| ignore  | bool   | If false the test body must match the test results.  If false, the results must not match the body. |
| message | String | Test comment, output on test failure. |
| body    | Object | JSON encoded output of hcl::from_str or the error message produced. The parsed HCL is checked against this for success. |

To bootstrap tests, if `<test-name>.hcl.json` does not exist or fails to parse the test will be ignored and it will dump the parsed HCL file as JSON.  Which can be verified and turned into the JSON file.