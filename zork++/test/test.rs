use clap::Parser;
use color_eyre::Result;
use std::{fs, path::Path};
use tempfile::tempdir;
use zork::cli::input::CliArgs;

#[test]
// #[ignore] // Ignore because the Clang's version in the VM's isn't newer enough to not support module partitions
fn test_clang_full_process() -> Result<()> {
    let temp = tempdir()?;

    assert!(zork::worker::run_zork(
        &CliArgs::parse_from([
            "",
            "new",
            "clang_example",
            "--compiler",
            "clang",
            "--template",
            "basic"
        ]),
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
// #[ignore]
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
#[ignore] // Provisional
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

mod local_env_tests {
    use super::*;
    use std::env;

    /// This test allows the developers to specify a path in local environments, having the opportunity
    /// to debug the Zork++ source code from a concrete location.
    ///
    /// For example, we can use the `[Zero project source code](https://github.com/zerodaycode/Zero)`
    /// in our local machines to debug the changes that we are making to Zork++ in real time,
    /// so by specifying a path, we allow Zork++ to start it's job in another concrete location,
    /// as if the binary where called from the specified path, and by running this test we can
    /// use a debugger to figure out what our changes are doing and how are affecting the codebase.
    #[test]
    #[ignore]
    fn test_clang_full_process_manually_by_specifying_the_project_root_on_linux() -> () {
        // Using env::home_dir because this test should be Unix specific
        // For any developer, change the path to whatever C++ project based on modules
        // you want to test Zork++ against
        #[allow(deprecated)]
        let mut path = env::home_dir().unwrap();
        path.push("code");
        path.push("c++");
        path.push("Zero");
        let process = zork::worker::run_zork(
            &CliArgs::parse_from([
                "",
                "-vv",
                "--root",
                &path.display().to_string(),
                "--match-files",
                "local_linux",
                "run",
            ]),
            &path,
        );
        assert!(process.is_ok());
    }
}
