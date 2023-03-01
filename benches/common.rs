use criterion::{measurement::Measurement, BenchmarkGroup, SamplingMode};
use std::fs;
use std::io;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Test {
    pub id: String,
    pub input: String,
}

pub fn load_tests() -> Result<Vec<Test>, io::Error> {
    let mut tests = Vec::new();

    for entry in fs::read_dir("testdata")? {
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

pub fn for_each_test<M, F>(group: &mut BenchmarkGroup<M>, tests: &[Test], f: F)
where
    M: Measurement,
    F: Fn(&mut BenchmarkGroup<M>, &Test),
{
    for test in tests {
        let (sampling_mode, measurement_time) = if test.id == "medium" || test.id == "large" {
            (SamplingMode::Flat, Duration::from_secs(5))
        } else {
            (SamplingMode::Auto, Duration::from_secs(2))
        };

        group.sampling_mode(sampling_mode);
        group.measurement_time(measurement_time);
        group.warm_up_time(measurement_time / 2);

        f(group, test);
    }
}
