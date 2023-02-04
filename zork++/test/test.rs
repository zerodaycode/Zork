use clap::Parser;
use color_eyre::Result;
use std::{fs, path::Path};
use tempfile::tempdir;
use zork::cli::input::CliArgs;

#[test]
fn test_clang_full_process() -> Result<()> {
    let temp = tempdir()?;

    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "new", "clang_example", "--compiler", "clang"]),
        Path::new(temp.path())
    )
    .is_ok());

    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "-vv", "run"]),
        Path::new(temp.path())
    )
    .is_ok());

    Ok(temp.close()?)
}

#[cfg(target_os = "windows")]
#[test]
fn test_msvc_full_process() -> Result<()> {
    let temp = tempdir()?;

    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "new", "msvc_example", "--compiler", "msvc"]),
        Path::new(temp.path())
    )
    .is_ok());

    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "-vv", "run"]),
        Path::new(temp.path())
    )
    .is_ok());

    Ok(temp.close()?)
}

#[cfg(target_os = "windows")]
#[test]
fn test_gcc_windows_full_process() -> Result<()> {
    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "new", "gcc_example", "--compiler", "gcc"]),
        Path::new(".") // Unable to run GCC tests because the gcm.cache folder, that
                       // we just wasn't able to discover how to specify a directory for it
    )
    .is_ok());

    assert!(
        zork::worker::run_zork(&CliArgs::parse_from(["", "-vv", "run"]), Path::new(".")).is_ok()
    );

    // Clearing the GCC dirs
    fs::remove_dir_all("./gcc_example")?;
    fs::remove_dir_all("./gcm.cache")?;
    fs::remove_dir_all("./out")?;

    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn test_gcc_linux_full_process() -> Result<()> {
    let temp = tempdir()?;

    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "new", "gcc_example", "--compiler", "gcc"]),
        Path::new(temp.path())
    )
    .is_ok());

    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "-vv", "run"]),
        Path::new(temp.path())
    )
    .is_ok());

    Ok(temp.close()?)
}

#[test]
fn test_full_program_with_multi_config_files() -> Result<()> {
    let temp = tempdir()?;

    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "new", "clang_example", "--compiler", "clang"]),
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

    // GCC specifics
    let gcc_path_by_os = if cfg!(target_os = "windows") {
        Path::new(".")
    } else {
        temp.path()
    };

    assert!(zork::worker::run_zork(
        &CliArgs::parse_from(["", "new", "gcc_example", "--compiler", "gcc"]),
        gcc_path_by_os
    )
    .is_ok());
    assert!(
        zork::worker::run_zork(&CliArgs::parse_from(["", "-vv", "run"]), gcc_path_by_os).is_ok()
    );

    // Clearing the GCC dirs
    fs::remove_dir_all("./gcc_example")?;
    fs::remove_dir_all("./gcm.cache")?;
    fs::remove_dir_all("./out")?;

    Ok(temp.close()?)
}
