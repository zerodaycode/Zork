use clap::Parser;
use color_eyre::Result;
use std::path::Path;
use tempfile::tempdir;
use zork::cli::input::CliArgs;

#[test]
fn test_full_program_with_three_config_files() -> Result<()> {
    let temp = tempdir()?;

    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "new", "clang_example", "--compiler", "clang"]),
        Path::new(temp.path())
    )
    .is_ok());

    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "new", "gcc_example", "--compiler", "gcc"]),
        Path::new(temp.path())
    )
    .is_ok());

    if cfg!(target_os = "windows") {
        assert!(zork::worker::run_zork(
            &CliArgs::parse_from(["", "new", "msvc_example", "--compiler", "msvc"]),
            Path::new(temp.path())
        )
        .is_ok());
    }

    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "-vv", "run"]),
        Path::new(temp.path())
    )
    .is_ok());

    Ok(temp.close()?)
}
