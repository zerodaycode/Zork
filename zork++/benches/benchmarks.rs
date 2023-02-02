//! Benchmarks tests for measuring the performance of the code

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::Path;
use zork::{
    cache::{self, ZorkCache},
    compiler::build_project,
    config_file::ZorkConfigFile,
    utils::reader::{build_model, find_config_file},
};

/// TODO See docs above before documenting this. Also, this is
/// the preliminar implementation, but we must difference this
/// tasks by also launching the command lines thorugh shells or
/// by just generating their code (the obviously optimal)
pub fn build_project_benchmark(c: &mut Criterion) {
    let config_file: String = find_config_file(Path::new("."))
        .expect("Failed to find a configuration file for the benchmarks");
    let config: ZorkConfigFile = toml::from_str(config_file.as_str())
        .expect("Error parsing the benchmarks Zork configuration file");

    let program_data = build_model(&config);

    c.bench_function("Build project", |b| {
        b.iter(|| {
            build_project(
                black_box(&program_data),
                black_box(&ZorkCache::default()),
                false,
            )
        })
    });

    c.bench_function("Cache loading time", |b| {
        b.iter(|| cache::load(black_box(&program_data)))
    });
}

criterion_group!(benches, build_project_benchmark);
criterion_main!(benches);
