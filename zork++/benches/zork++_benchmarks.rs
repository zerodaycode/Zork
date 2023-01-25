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

use criterion::Criterion;
use zork::compiler::build_project;

/// TODO See docs above before documenting this. Also, this is
/// the preliminar implementation, but we must difference this
/// tasks by also launching the command lines thorugh shells or 
/// by just generating their code (the obviously optimal)
pub fn build_project_benchmark(c: &mut Criterion)