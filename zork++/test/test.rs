use std::path::Path;

use clap::Parser;
use zork::cli::input::CliArgs;

/// This should run being the test directory the base path
/// Should found three config files, available at the root also
/// TODO

#[test]
fn test_full_program_with_three_config_files() {
    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "new", "clang_example", "--compiler", "clang"]),
        Path::new("./test"))
        .is_ok());
    assert!(
        zork::worker::run_zork(
            &CliArgs::parse_from(["", "-vv", "run"]),
            Path::new("./test")
        ).is_ok());
}

#[test]
fn test_find_config_files() {
    assert!(
        !zork::utils::reader
            ::find_config_files(Path::new("./test"))
            .unwrap()
            .is_empty()
    )
}