#![warn(clippy::pedantic)]
#![allow(clippy::struct_excessive_bools)]

use anyhow::{Context as _, Result, anyhow, bail};
use clap::Parser;
use globset::{GlobBuilder, GlobMatcher};
use hcl::eval::{Context, Evaluate};
use hcl::structure::Body;
use rayon::prelude::*;
use serde_json::Value;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// Converts HCL to JSON.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A glob pattern to match files when recursively scanning directories.
    ///
    /// Required if any of the input paths is a directory. Ignored otherwise.
    #[arg(short, long)]
    glob: Option<String>,
    /// Read input into a map keyed by file path of the origin file.
    ///
    /// If multiple input files or at least one directory is provided, this reads the result into
    /// a map keyed by file path instead of an array.
    ///
    /// If only a single input file is provided or the input is passed via stdin, this option is
    /// has no effect.
    #[arg(short = 'P', long)]
    file_paths: bool,
    /// Pretty-print the resulting JSON.
    ///
    /// By default, compact JSON is emitted.
    #[arg(short, long)]
    pretty: bool,
    /// Continue on errors that occur while processing individual files.
    ///
    /// If the flag is provided, processing of the remaining input files continues after an error
    /// occurred. This is useful if you want to process files using a glob pattern and one of the
    /// files is malformed. In this case a warning is logged to stderr and the file is ignored.
    ///
    /// If only a single input file is provided or the input is passed via stdin, this option is
    /// has no effect.
    #[arg(short = 'C', long)]
    continue_on_error: bool,
    /// Attempt to simplify expressions which do not contain any variables or unknown functions.
    ///
    /// This will attempt to simplify expressions as much as possible. For example, the binary
    /// operation `1 + 1` can be simplified to just `2` because it does not contain any variables
    /// or unknown function calls.
    #[arg(short, long)]
    simplify: bool,
    /// Paths to read HCL files from.
    ///
    /// This can be files or directories. Directories require passing a glob pattern via --glob.
    paths: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let writer = BufWriter::new(io::stdout());

    if args.paths.is_empty() {
        convert(io::stdin(), writer, &args)
    } else {
        if args.paths.len() == 1 {
            let path = &args.paths[0];

            if path == &PathBuf::from("-") {
                return convert(io::stdin(), writer, &args);
            }

            if !path.is_dir() {
                return convert(File::open(path)?, writer, &args);
            }
        }

        let mut paths = Vec::with_capacity(args.paths.len());

        for path in &args.paths {
            if path.is_dir() {
                let Some(pattern) = &args.glob else {
                    bail!("--glob is required if directory arguments are specified")
                };

                let path_pattern = path.join(pattern);

                let matcher = GlobBuilder::new(&path_pattern.to_string_lossy())
                    .literal_separator(true)
                    .build()?
                    .compile_matcher();

                if let Err(err) = glob_files(&matcher, path, &mut paths) {
                    if !args.continue_on_error {
                        return Err(err);
                    }

                    eprintln!(
                        "Warning: directory `{}` ignored due to errors: {err}",
                        path.display()
                    );
                }
            } else {
                paths.push(path.clone());
            }
        }

        bulk_convert(&paths, writer, &args)
    }
}

#[inline]
fn convert<R: Read, W: Write>(reader: R, writer: W, args: &Args) -> Result<()> {
    let value = read_hcl(reader, args)?;
    write_json(writer, &value, args)
}

fn bulk_convert<W: Write>(paths: &[PathBuf], writer: W, args: &Args) -> Result<()> {
    let iter = paths.into_par_iter();

    let results = if args.continue_on_error {
        iter.filter_map(|path| match process_file(path, args) {
            Ok(result) => Some(result),
            Err(err) => {
                eprintln!(
                    "Warning: file `{}` ignored due to errors: {err}",
                    path.display()
                );
                None
            }
        })
        .collect::<Vec<_>>()
    } else {
        iter.map(|path| {
            process_file(path, args)
                .with_context(|| anyhow!("failed to process file `{}`", path.display()))
        })
        .collect::<Result<_>>()?
    };

    let value = if args.file_paths {
        Value::from_iter(results)
    } else {
        results.into_iter().map(|(_, value)| value).collect()
    };

    write_json(writer, &value, args)
}

#[inline]
fn process_file(path: &Path, args: &Args) -> Result<(String, Value)> {
    let file = File::open(path)?;
    let value = read_hcl(file, args)?;
    let path = path.display().to_string();
    Ok((path, value))
}

#[inline]
fn read_hcl<R: Read>(reader: R, args: &Args) -> Result<Value> {
    let reader = BufReader::new(reader);

    let value = if args.simplify {
        let mut body: Body = hcl::from_reader(reader)?;
        let ctx = Context::default();
        // Evaluate as much as possible and ignore errors.
        _ = body.evaluate_in_place(&ctx);
        hcl::from_body(body)?
    } else {
        hcl::from_reader(reader)?
    };

    Ok(value)
}

#[inline]
fn write_json<W: Write>(writer: W, value: &Value, args: &Args) -> Result<()> {
    if args.pretty {
        serde_json::to_writer_pretty(writer, value)?;
    } else {
        serde_json::to_writer(writer, value)?;
    }

    Ok(())
}

fn glob_files(matcher: &GlobMatcher, dir: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    for entry in walkdir::WalkDir::new(dir).sort_by_file_name() {
        let path = entry?.into_path();

        if path.is_file() && matcher.is_match(&path) {
            paths.push(path);
        }
    }

    Ok(())
}
