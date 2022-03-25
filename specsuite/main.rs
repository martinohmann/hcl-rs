use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Serialize, Deserialize)]
struct Test {
    #[serde(default)]
    ignore: bool,
    message: String,
    body: Value,
}

fn find_tests<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>, std::io::Error> {
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

    println!("running {} tests", tests.len());
    for test_file in tests {
        print!("test {} - ", test_file.to_str().expect("Invalid path"));

        let json_file = test_file.with_extension("hcl.json");
        print!("{} ... ", json_file.to_str().expect("Invalid path"));
        let test = std::fs::read_to_string(json_file)
            .ok()
            .and_then(|data| serde_json::from_str::<Test>(&data).ok());

        let hcl_content = std::fs::read_to_string(test_file)?;

        let result = match hcl::from_str::<Value>(&hcl_content) {
            Ok(value) => Some(value),
            Err(hcl::Error::Message { msg, location: _ }) => Some(json!({ "message": msg })),
            Err(err) => Some(json!({ "message": format!("{:#?}", err) })),
        };

        let (status, msg) = if test.is_none() {
            let dump = Test {
                ignore: false,
                message: "Auto-generated test. Verify for accuracy.".to_string(),
                body: result.unwrap(),
            };
            ("ignore", serde_json::to_string_pretty(&dump)?)
        } else {
            let test = test.unwrap();
            let result = result.unwrap();
            if test.body == result {
                if test.ignore {
                    ("fail", "Ignored test is now passing".to_string())
                } else {
                    ("ok", "success".to_string())
                }
            } else {
                let status = if test.ignore { "ignore" } else { "fail" };
                (status, format!("Found:\n{}\nExpected:\n{}",  serde_json::to_string_pretty(&result)?, serde_json::to_string_pretty(&test.body)?))
            }
        };

        match status {
            "ok" => {
                successes += 1;
                println!("\x1b[32mok\x1b[0m");
            }
            "fail" => {
                failures += 1;
                println!("\x1b[31mfail\x1b[0m\n{}", msg);
            }
            "ignore" => {
                ignored += 1;
                println!("\x1b[33mignored\x1b[0m\n{}", msg);
            }
            _ => {
                unreachable!("Status must be ok, fail, or ignore");
            }
        }
    }

    let status = if failures == 0 {
        "\x1b[32mok\x1b[0m"
    } else {
        "\x1b[31mfailed\x1b[0m"
    };
    println!(
        "\ntest result: {}. {} passed; {} failed; {} ignored; finished in {:.2}s\n",
        status,
        successes,
        failures,
        ignored,
        timer.elapsed().as_secs_f64()
    );

    if failures == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
