//! Benchmarks tests for measuring the performance of the code

use clap::Parser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{fs, path::Path};
use zork::{
    cache::{self, ZorkCache},
    cli::input::CliArgs,
    compiler::build_project,
    config_file::ZorkConfigFile,
    utils::reader::{build_model, find_config_files},
    worker::run_zork,
};

pub fn build_project_benchmark(c: &mut Criterion) {
    let config_files = find_config_files(Path::new("."))
        .expect("Failed to find a configuration file for the benchmarks");

    let raw_file = fs::read_to_string(&config_files.get(0).unwrap().path).unwrap();
    let config: ZorkConfigFile = toml::from_str(raw_file.as_str()).unwrap();
    let program_data = build_model(&config);

    c.bench_function(
        "[Mocked Main - Build] - Test a full execution of the program",
        |b| b.iter(|| run_zork(&CliArgs::parse_from(["", "-vv", "run"]), Path::new("."))),
    );

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
