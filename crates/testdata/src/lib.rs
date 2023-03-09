use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SUPPORTED_EXTS: [&str; 2] = ["hcl", "tf"];

fn has_supported_ext(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str().map(|ext| SUPPORTED_EXTS.contains(&ext)))
        .unwrap_or(false)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Test {
    pub path: PathBuf,
    pub input: String,
}

impl Test {
    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_string_lossy().to_string()
    }
}

pub fn load() -> io::Result<Vec<Test>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut tests = Vec::new();

    for entry in fs::read_dir(manifest_dir.join("data"))? {
        let path = entry?.path();

        if path.is_file() && has_supported_ext(&path) {
            let input = fs::read_to_string(&path)?;
            tests.push(Test { path, input });
        }
    }

    tests.sort();

    Ok(tests)
}
