use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::read_to_string as read;

#[test]
fn small() {
    Command::cargo_bin("hcl2json")
        .unwrap()
        .arg("../testdata/data/small.tf")
        .assert()
        .success()
        .stdout(read("tests/fixtures/small.json").unwrap());
}

#[test]
fn small_pretty() {
    Command::cargo_bin("hcl2json")
        .unwrap()
        .args(["../testdata/data/small.tf", "--pretty"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/small.pretty.json").unwrap());
}

#[test]
fn glob_required_for_dirs() {
    Command::cargo_bin("hcl2json")
        .unwrap()
        .arg("../testdata")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--glob is required if directory arguments are specified",
        ));
}

#[test]
fn glob_array() {
    Command::cargo_bin("hcl2json")
        .unwrap()
        .args([
            "../testdata/data",
            "--pretty",
            "--glob",
            "**/{small,medium}.tf",
        ])
        .assert()
        .success()
        .stdout(read("tests/fixtures/glob.array.json").unwrap());
}

#[test]
fn glob_map() {
    Command::cargo_bin("hcl2json")
        .unwrap()
        .args([
            "../testdata/data",
            "--pretty",
            "--glob",
            "**/{small,medium}.tf",
            "--file-paths",
        ])
        .assert()
        .success()
        .stdout(read("tests/fixtures/glob.map.json").unwrap());
}

#[test]
fn glob_continue_on_error() {
    Command::cargo_bin("hcl2json")
        .unwrap()
        .args([
            "../testdata/data",
            "--pretty",
            "--glob",
            "{small,README}.*",
            "--file-paths",
            "--continue-on-error",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Warning: File `../testdata/data/README.md` skipped due to error:",
        ))
        .stdout(read("tests/fixtures/glob.continue-on-error.json").unwrap());
}

#[test]
fn glob_array_no_match() {
    Command::cargo_bin("hcl2json")
        .unwrap()
        .args(["../testdata/data", "--pretty", "--glob", "*never-matches"])
        .assert()
        .success()
        .stdout("[]");
}

#[test]
fn glob_map_no_match() {
    Command::cargo_bin("hcl2json")
        .unwrap()
        .args([
            "../testdata/data",
            "--pretty",
            "--glob",
            "*never-matches",
            "--file-paths",
        ])
        .assert()
        .success()
        .stdout("{}");
}
