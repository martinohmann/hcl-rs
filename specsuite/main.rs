use serde_json::{json, Value};
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

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
    for test in tests {
        print!("test {} - ", test.to_str().expect("Invalid path"));

        let json = test.with_extension("hcl.json");
        print!("{} ... ", json.to_str().expect("Invalid path"));
        let json = if json.is_file() {
            let data = std::fs::read_to_string(json)?;
            let value: Value = serde_json::from_str(&data)?;
            Some(value)
        } else {
            None
        };

        let contents = std::fs::read_to_string(test)?;
        match hcl::from_str::<Value>(&contents) {
            Ok(value) => {
                if json.is_none() {
                    ignored += 1;
                    println!("\x1b[33mignored\x1b[0m\n{}", serde_json::to_string_pretty(&value)?);
                } else if json.as_ref() == Some(&value) {
                    successes += 1;
                    println!("\x1b[32mok\x1b[0m");
                } else {
                    failures += 1;
                    println!("\x1b[31mfail\x1b[0m\nFound: {:?}\nExpect: {:?}", json, value);
                }
            }
            Err(hcl::Error::Message { msg, location: _ }) => {
                let value = Some(json!({ "Message": msg }));
                if json.is_none() {
                    ignored += 1;
                    println!("\x1b[33mignored\x1b[0m\n{}", serde_json::to_string_pretty(&value)?);
                } else if json == value {
                    successes += 1;
                    println!("\x1b[32mok\x1b[0m");
                } else {
                    failures += 1;
                    println!("\x1b[31mfail\x1b[0m\nFound: {:?}\nExpect: {:?}", value, json);
                }
            }
            Err(msg) => {
                failures += 1;
                println!("\x1b[31mfail\x1b[0m\n{:?}", msg);
            }
        };
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
