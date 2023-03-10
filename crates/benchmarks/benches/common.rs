use criterion::{measurement::Measurement, BenchmarkGroup, SamplingMode};
use std::time::Duration;
use testdata::Test;

pub fn for_each_test<M, F>(group: &mut BenchmarkGroup<M>, tests: &[Test], f: F)
where
    M: Measurement,
    F: Fn(&mut BenchmarkGroup<M>, &Test),
{
    for test in tests {
        let input_len = test.input.len();

        let (sampling_mode, measurement_time) = if input_len > 4096 {
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
