//! Benchmarks tests for measuring the performance of the code

use std::path::Path;

use clap::Parser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use zork::{
    cache::{self, ZorkCache},
    cli::input::CliArgs,
    config_file::{self, ZorkConfigFile},
    utils::{self, reader::build_model},
};
use zork::compiler::generate_commands;

pub fn build_project_benchmark(c: &mut Criterion) {
    let config: ZorkConfigFile =
        config_file::zork_cfg_from_file(utils::constants::CONFIG_FILE_MOCK).unwrap();
    let cli_args = CliArgs::parse();
    let program_data = build_model(config, &cli_args, Path::new(".")).unwrap();

    c.bench_function("Generate commands", |b| {
        b.iter(|| generate_commands(black_box(&program_data), black_box(ZorkCache::default()), &cli_args))
    });

    c.bench_function("Cache loading time", |b| {
        b.iter(|| cache::load(black_box(&program_data), &CliArgs::default()))
    });
}

criterion_group!(benches, build_project_benchmark);
criterion_main!(benches);
