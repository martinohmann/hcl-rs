#![warn(clippy::pedantic)]

use anyhow::{Result, bail};
use clap::Parser;
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
    #[arg(short, long)]
    glob: Option<String>,
    /// Pretty-print the resulting JSON.
    #[arg(short, long)]
    pretty: bool,
    /// Attempt to simply expressions which don't contain any variables or unknown functions.
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

                glob_files(path, pattern, &mut paths)?;
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

fn bulk_convert<W: Write>(paths: &[PathBuf], mut writer: W, args: &Args) -> Result<()> {
    if paths.is_empty() {
        writer.write_all(b"{}")?;
        return Ok(());
    }

    let results = paths
        .into_par_iter()
        .map(|path| {
            let value = read_hcl(File::open(path)?, args)?;
            let path = path.display().to_string();
            Ok((path, value))
        })
        .collect::<Result<Vec<_>>>()?;

    let value = Value::from_iter(results);
    write_json(writer, &value, args)
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

fn glob_files(path: &Path, pattern: &str, paths: &mut Vec<PathBuf>) -> Result<()> {
    let path_pattern = path.join(pattern);

    for result in glob::glob(&path_pattern.to_string_lossy())? {
        let path = result?;
        if path.is_file() {
            paths.push(path);
        }
    }

    Ok(())
}
