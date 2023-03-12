//! Benchmarks tests for measuring the performance of the code

use std::cell::RefCell;
use std::rc::Rc;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use zork::{
    cache::{self, ZorkCache},
    cli::input::CliArgs,
    compiler::build_project,
    config_file::ZorkConfigFile,
    utils::{self, reader::build_model},
};

pub fn build_project_benchmark(c: &mut Criterion) {
    let config: ZorkConfigFile = toml::from_str(utils::constants::CONFIG_FILE_MOCK).unwrap();
    let program_data = build_model(&config);

    c.bench_function("Build project", |b| {
        b.iter(|| build_project(black_box(&program_data), black_box(Rc::new(RefCell::new(ZorkCache::default()))), false))
    });

    c.bench_function("Cache loading time", |b| {
        b.iter(|| cache::load(black_box(&program_data), &CliArgs::default()))
    });
}

criterion_group!(benches, build_project_benchmark);
criterion_main!(benches);
