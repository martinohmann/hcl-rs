use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Test {
    pub id: String,
    pub input: String,
}

pub fn load() -> Result<Vec<Test>, io::Error> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut tests = Vec::new();

    for entry in fs::read_dir(manifest_dir.join("data"))? {
        let path = entry?.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "hcl") {
            let input = fs::read_to_string(&path)?;
            let id = path.file_stem().unwrap().to_string_lossy().to_string();
            tests.push(Test { id, input });
        }
    }

    tests.sort();

    Ok(tests)
}
