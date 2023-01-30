//! Benchmarks tests for measuring the performance of the code
//!
//! There are a lot of things to count on them. For example, the
//! compiler might take to process a file an incredible amount of
//! time, and ruin the overall of the benchmarks.
//!
//! We mean, for example, build_project nowadays also executes the commands
//! generated. We must decouple this things and measure effectivly
//! our processes. In any case, most of the time there are system
//! call involved, but this benches are more for measure the impact
//! of changes or future changes on the code.
//!
//! Also, this doens't mean that we musn't benchmark the full process.
//! For sure it will be worth, even depending on external

use std::path::Path;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use zork::{compiler::build_project, cache::{ZorkCache, self}, utils::reader::{find_config_file, build_model}, config_file::ZorkConfigFile};

/// TODO See docs above before documenting this. Also, this is
/// the preliminar implementation, but we must difference this
/// tasks by also launching the command lines thorugh shells or
/// by just generating their code (the obviously optimal)
pub fn build_project_benchmark(c: &mut Criterion) {

    let config_file: String =
        find_config_file(Path::new("."))
            .expect("Failed to find a configuration file for the benchmarks");
    let config: ZorkConfigFile = toml::from_str(config_file.as_str())
        .expect("Error parsing the benchmarks Zork configuration file");

    let program_data = build_model(&config);

    c.bench_function(
        "Build project",
        |b| b.iter(
            || build_project(
                black_box(&program_data),
                black_box(&ZorkCache::default()),
                false
            ) 
        )
    );

    c.bench_function(
        "Cache loading time",
        |b| b.iter(
            || cache::load(
                black_box(&program_data)
            ) 
        )
    );
}

criterion_group!(benches, build_project_benchmark);
criterion_main!(benches);
