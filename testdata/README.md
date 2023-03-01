# testdata

This directory contains input files that are used in tests and benchmarks.

The file `large.hcl` was generated from a local checkout of the
[`terraform-aws-modules/terraform-aws-eks`](https://github.com/terraform-aws-modules/terraform-aws-eks)
repository at commit
[`c7565e265e23cbc622041fc153193f490cfe948f`](https://github.com/terraform-aws-modules/terraform-aws-eks/tree/c7565e265e23cbc622041fc153193f490cfe948f)
by running:

```sh
find . -name '*.tf' -maxdepth 1 -exec cat {} + > ../large.hcl
```

Copyright of the contents of this file belongs to the original owners.
