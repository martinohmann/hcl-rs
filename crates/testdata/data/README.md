# testdata

This directory contains input files that are used in tests and benchmarks.

The file `medium.tf` is a copy of `main.tf` from the
[`terraform-aws-modules/terraform-aws-ec2-instance`](https://github.com/terraform-aws-modules/terraform-aws-ec2-instance)
repository at commit
[`baf00599be6be3da99992634db6289f99071d45a`](https://github.com/terraform-aws-modules/terraform-aws-ec2-instance/blob/baf00599be6be3da99992634db6289f99071d45a/main.tf).

The file `large.tf` was generated from a local checkout of the
[`terraform-aws-modules/terraform-aws-eks`](https://github.com/terraform-aws-modules/terraform-aws-eks)
repository at commit
[`c7565e265e23cbc622041fc153193f490cfe948f`](https://github.com/terraform-aws-modules/terraform-aws-eks/tree/c7565e265e23cbc622041fc153193f490cfe948f)
by running:

```sh
find . -name '*.tf' -maxdepth 1 -exec cat {} + > ../large.tf
```

Copyright of the contents of these files belongs to the original owners.
