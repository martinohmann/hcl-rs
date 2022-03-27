use assert_json_diff::{assert_json_matches_no_panic, CompareMode, Config};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Serialize, Deserialize)]
struct Test {
    #[serde(default)]
    ignore: bool,
    message: String,
    body: Value,
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

fn main() -> Result<(), Box<dyn Error>> {
    let timer = Instant::now();
    let mut failures = 0;
    let mut ignored = 0;
    let mut successes = 0;

    let mut tests = find_tests("specsuite/")?;
    tests.sort();

    println!("\nrunning {} tests", tests.len());

    for test_file in tests {
        let json_file = test_file.with_extension("hcl.json");

        print!(
            "test {} - {} ... ",
            test_file.to_string_lossy(),
            json_file.to_string_lossy()
        );

        let expected = fs::read_to_string(json_file)
            .ok()
            .and_then(|data| serde_json::from_str::<Test>(&data).ok());

        let hcl_content = fs::read_to_string(test_file)?;

        let result = match hcl::from_str(&hcl_content) {
            Ok(value) => value,
            Err(hcl::Error::Message { msg, .. }) => json!({ "message": msg }),
            Err(err) => json!({ "message": format!("{:#?}", err) }),
        };

        let (status, msg) = match expected {
            Some(expected) => {
                if expected.body == result {
                    if expected.ignore {
                        (Status::Failed, "Ignored test is now passing".to_string())
                    } else {
                        (Status::Ok, String::new())
                    }
                } else {
                    let msg = format!(
                        "Comment: {}\n{}",
                        expected.message,
                        assert_json_matches_no_panic(
                            &expected.body,
                            &result,
                            Config::new(CompareMode::Strict)
                        ).unwrap_err()
                    );

                    if expected.ignore {
                        (Status::Ignored, msg)
                    } else {
                        (Status::Failed, msg)
                    }
                }
            }
            None => {
                let dump = Test {
                    ignore: false,
                    message: "Auto-generated test. Verify for accuracy.".to_string(),
                    body: result,
                };
                (Status::Ignored, serde_json::to_string_pretty(&dump)?)
            }
        };

        match status {
            Status::Ok => {
                successes += 1;
                println!("{}", status);
            }
            Status::Failed => {
                failures += 1;
                println!("{}\n{}", status, textwrap::indent(&msg, "  "));
            }
            Status::Ignored => {
                ignored += 1;
                println!("{}\n{}", status, textwrap::indent(&msg, "  "));
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
