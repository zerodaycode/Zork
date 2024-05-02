use clap::Parser;
use color_eyre::Result;
use std::fs;
use tempfile::tempdir;
use zork::cli::input::CliArgs;

#[test]
fn test_clang_full_process() -> Result<()> {
    let tempdir = tempdir()?;
    let path = tempdir.path().to_str().unwrap();
    env_logger::init();

    assert!(zork::worker::run_zork(&CliArgs::parse_from([
        "",
        "--root",
        path, // TODO: pass this path directly to the generated zork++ cfg template file
        "new",
        "clang_example",
        "--compiler",
        "clang",
        "--template",
        "basic",
    ]),)
    .is_ok());

    let process_result = zork::worker::run_zork(&CliArgs::parse_from([
        "", "-vv", "--root", path,
        /* "--driver-path",
        "clang++-16", // Local cfg issues */
        "run",
    ]));
    assert!(process_result.is_ok(), "{}", process_result.unwrap_err());

    Ok(tempdir.close()?)
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
        &CliArgs::parse
        Path::new(".") // Unable to run GCC tests because the gcm.cache folder, that
                       // we just wasn't able to discover how to specify a directory for it
    )
    .is_ok());

    assert!(
        zork::worker::run_zork(&CliArgs::parse_from(["", "-vv", "run"]), Path::new(".")).is_ok()
    );

    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
/*
In the GitHub's virtual machines, we are still unable, due
to the gcm.cache path.

cc1plus: fatal error: iostream: No such file or directory
compilation terminated.
In module imported at /tmp/.tmpGaFLnR/gcc_example/main.cpp:8:5:
/usr/include/c++/13.2.1/iostream: error: failed to read compiled module: No such file or directory
/usr/include/c++/13.2.1/iostream: note: compiled module file is ‘gcm.cache/./usr/include/c++/13.2.1/iostream.gcm’
/usr/include/c++/13.2.1/iostream: note: imports must be built before being imported
/usr/include/c++/13.2.1/iostream: fatal error: returning to the gate for a mechanical issue
compilation terminated.
 */
fn test_gcc_full_process() -> Result<()> {
    let tempdir = tempdir()?;
    let path = tempdir.path().to_str().unwrap();

    assert!(zork::worker::run_zork(&CliArgs::parse_from([
        "",
        "--root",
        path,
        "new",
        "gcc_example",
        "--compiler",
        "gcc",
    ]),)
    .is_ok());

    assert!(
        zork::worker::run_zork(&CliArgs::parse_from(["", "-vv", "--root", path, "run"]),).is_ok()
    );

    // Clearing the GCC modules cache (weird, isn't generated at the invoked project's root)
    // maybe we should change dir? but that collide with the purpose of specifiying the project
    // root clearly
    fs::remove_dir_all("./gcm.cache")?;

    Ok(tempdir.close()?)
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
    fn test_local_clang_full_process_manually_by_specifying_the_project_root_on_linux() {
        // Using env::home_dir because this test should be Unix specific
        // For any developer, change the path to whatever C++ project based on modules
        // you want to test Zork++ against
        #[allow(deprecated)]
        let mut path = env::home_dir().unwrap();
        path.push("code");
        path.push("c++");
        path.push("Zero");

        let process = zork::worker::run_zork(&CliArgs::parse_from([
            "",
            "-vv",
            "--root",
            &path.display().to_string(),
            "--driver-path",
            "clang++-16",
            "--match-files",
            "local_linux",
            "run",
        ]));
        assert!(process.is_ok());
    }
}
