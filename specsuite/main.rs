use assert_json_diff::{assert_json_matches_no_panic, CompareMode, Config};
use hcl::eval::Context;
use hcl::{Map, Value};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Deserialize, Default)]
struct Hcldec {
    #[serde(default)]
    ignore: bool,
    variables: Map<String, Value>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Outcome {
    Result(Value),
    Diagnostics { error: String },
}

enum Status {
    Ok,
    Failed,
    Ignored,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Ok => f.write_str("\x1b[32mok\x1b[0m"),
            Status::Failed => f.write_str("\x1b[31mFAILED\x1b[0m"),
            Status::Ignored => f.write_str("\x1b[33mignored\x1b[0m"),
        }
    }
}

fn find_tests<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>, io::Error> {
    let mut tests = Vec::new();

    let root = root.as_ref();

    if root.is_dir() {
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                tests.extend(find_tests(path)?);
            } else if path.is_file() && path.extension() == Some(OsStr::new("hcl")) {
                tests.push(path);
            }
        }
    }

    Ok(tests)
}

fn run_test(hcl_file: &PathBuf) -> Result<(Status, String), hcl::Error> {
    let t_file = hcl_file.with_extension("t");
    let hcldec_file = hcl_file.with_extension("hcldec");

    print!(
        "test {} - {} ... ",
        hcl_file.to_string_lossy(),
        t_file.to_string_lossy()
    );

    let hcldec = if hcldec_file.exists() {
        let content = fs::read_to_string(hcldec_file)?;
        hcl::from_str(&content)?
    } else {
        Hcldec::default()
    };

    let data = fs::read_to_string(hcl_file)?;

    let mut ctx = Context::new();
    for (name, value) in hcldec.variables {
        ctx.declare_var(name, value);
    }

    let result = match hcl::eval::from_str::<Value>(&data, &ctx) {
        Ok(value) => Outcome::Result(value),
        Err(err) => Outcome::Diagnostics {
            error: err.to_string(),
        },
    };

    if !t_file.exists() {
        return Ok((
            Status::Ignored,
            format!(
                "// Auto-generated test. Verify for accuracy.\n{}",
                hcl::to_string(&result)?,
            ),
        ));
    }

    let data = fs::read_to_string(t_file)?;
    let expected: Outcome = hcl::from_str(&data)?;

    let (status, msg) = if expected == result {
        if hcldec.ignore {
            (Status::Failed, "Ignored test is now passing".to_string())
        } else {
            (Status::Ok, String::new())
        }
    } else {
        let diff =
            assert_json_matches_no_panic(&expected, &result, Config::new(CompareMode::Strict))
                .unwrap_err();

        if hcldec.ignore {
            (Status::Ignored, diff)
        } else {
            (Status::Failed, diff)
        }
    };

    Ok((status, msg))
}

fn print_msg(status: Status, msg: &str) {
    println!("{}\n{}", status, textwrap::indent(msg, "  "))
}

fn main() -> Result<(), Box<dyn Error>> {
    let timer = Instant::now();
    let mut failures = 0;
    let mut ignored = 0;
    let mut successes = 0;

    let mut tests = find_tests("specsuite/")?;
    tests.sort();

    println!("\nrunning {} tests", tests.len());

    for hcl_file in tests {
        match run_test(&hcl_file) {
            Ok((status, msg)) => match status {
                Status::Ok => {
                    successes += 1;
                    println!("{}", status);
                }
                Status::Failed => {
                    failures += 1;
                    print_msg(status, &msg);
                }
                Status::Ignored => {
                    ignored += 1;
                    print_msg(status, &msg);
                }
            },
            Err(err) => {
                failures += 1;
                print_msg(Status::Failed, &err.to_string());
            }
        }
    }

    let (code, status) = if failures == 0 {
        (0, Status::Ok)
    } else {
        (1, Status::Failed)
    };

    println!(
        "\ntest result: {}. {} passed; {} failed; {} ignored; finished in {:.2}s\n",
        status,
        successes,
        failures,
        ignored,
        timer.elapsed().as_secs_f64()
    );

    std::process::exit(code);
}
