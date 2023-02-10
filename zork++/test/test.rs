use clap::Parser;
use color_eyre::Result;
use std::{fs, path::Path};
use tempfile::tempdir;
use zork::cli::input::CliArgs;

#[test]
#[ignore]
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
#[ignore]
/*
In Manjaro, I am able to fully run this test using tempdir.

But, in the GitHub's virtual machines, we are still unable, due
to the gcm.cache path.

/usr/include/c++/11/iostream: error: failed to write compiled module: No such file or directory
/usr/include/c++/11/iostream: note: compiled module file is ‘gcm.cache/./usr/include/c++/11/iostream.gcm’
In module imported at /tmp/.tmphw2WuM/gcc_example/main.cpp:8:5:
/usr/include/c++/11/iostream: error: failed to read compiled module: No such file or directory
/usr/include/c++/11/iostream: note: compiled module file is ‘gcm.cache/./usr/include/c++/11/iostream.gcm’
In module imported at /tmp/.tmp9Vk2YO/gcc_example/main.cpp:8:5:
/usr/include/c++/11/iostream: error: failed to read compiled module: No such file or directory
/usr/include/c++/11/iostream: note: compiled module file is ‘gcm.cache/./usr/include/c++/11/iostream.gcm’
/usr/include/c++/11/iostream: note: imports must be built before being imported
/usr/include/c++/11/iostream: fatal error: returning to the gate for a mechanical issue
compilation terminated.
/usr/include/c++/11/iostream: note: imports must be built before being imported
/usr/include/c++/11/iostream: fatal error: returning to the gate for a mechanical issue
 */
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
#[ignore]
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
    if cfg!(target_os = "windows") {
        assert!(zork::worker::run_zork(
            &CliArgs::parse_from(["", "new", "gcc_example", "--compiler", "gcc"]),
            Path::new(".")
        )
        .is_ok());
        assert!(
            zork::worker::run_zork(&CliArgs::parse_from(["", "-vv", "run"]), Path::new("."))
                .is_ok()
        );

        fs::remove_dir_all("./gcc_example")?;
        fs::remove_dir_all("./gcm.cache")?;
        fs::remove_dir_all("./out")?;
    }

    Ok(temp.close()?)
}
