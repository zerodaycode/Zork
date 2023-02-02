//! Benchmarks tests for measuring the performance of the code

use clap::Parser;
use color_eyre::{eyre::Context, Result};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{fs, path::Path};
use zork::{
    cache::{self, ZorkCache},
    cli::{
        input::{CliArgs, Command},
        output::commands::{self, autorun_generated_binary},
    },
    compiler::build_project,
    config_file::ZorkConfigFile,
    project_model::{compiler::CppCompiler, ZorkModel},
    utils::{
        self,
        reader::{build_model, find_config_files, ConfigFile},
        template::create_templated_project,
    },
};

/// TODO See docs above before documenting this. Also, this is
/// the preliminar implementation, but we must difference this
/// tasks by also launching the command lines thorugh shells or
/// by just generating their code (the obviously optimal)
pub fn build_project_benchmark(c: &mut Criterion) {
    let config_files = find_config_files(Path::new("."))
        .expect("Failed to find a configuration file for the benchmarks");
    let raw_file = fs::read_to_string(&config_files.get(0).unwrap().path).unwrap();

    let config: ZorkConfigFile = toml::from_str(raw_file.as_str()).unwrap();

    let program_data = build_model(&config);

    c.bench_function(
        "[Mocked Main - Build] - Test a full execution of the program",
        |b| b.iter(|| mocked_main(black_box("build"))),
    );

    c.bench_function("Build project", |b| {
        b.iter(|| {
            build_project(
                black_box(&program_data),
                black_box(&ZorkCache::default()),
                false,
            )
        })
    });

    c.bench_function("Cache loading time", |b| {
        b.iter(|| cache::load(black_box(&program_data)))
    });
}

criterion_group!(benches, build_project_benchmark);
criterion_main!(benches);

/// This is a direct copy paste for the code generated for the entry point of the
/// program, in order to measure the overall performance of the full execution of the project.
fn mocked_main(mocked_cli_arg: &str) -> Result<()> {
    color_eyre::install()?;

    let cli_args = CliArgs::parse_from(["", mocked_cli_arg]);

    let config_files: Vec<ConfigFile> = find_config_files(Path::new("."))
        .with_context(|| "We didn't found a `zork.toml` configuration file")?;

    for config_file in config_files {
        let raw_file = fs::read_to_string(config_file.path)
            .with_context(|| {
                format!(
                    "An error happened parsing the configuration file: {:?}",
                    config_file.dir_entry.file_name()
                )
            })
            .unwrap();

        let config: ZorkConfigFile = toml::from_str(raw_file.as_str())
            .with_context(|| "Could not parse configuration file")?;

        let program_data = build_model(&config);
        create_output_directory(Path::new("."), &program_data)?;

        let cache =
            cache::load(&program_data).with_context(|| "Unable to load the Zork++ caché")?;

        do_main_work_based_on_cli_input(&cli_args, &program_data, &cache).with_context(|| {
            format!(
                "Failed to build the project for the config file: {:?}",
                config_file.dir_entry.file_name()
            )
        })?;

        cache::save(&program_data, cache)?;
    }

    Ok(())
}

/// Helper for reduce the cyclomatic complextity of the main fn.
///
/// Contains the main calls to the generation of the compilers commands lines,
/// the calls to the process that runs those ones, the autorun the generated
/// binaries, the tests declared for the projects...
fn do_main_work_based_on_cli_input(
    cli_args: &CliArgs,
    program_data: &ZorkModel,
    cache: &ZorkCache,
) -> Result<()> {
    match cli_args.command {
        Command::Build => {
            let commands = build_project(program_data, cache, false)
                .with_context(|| "Failed to build project")?;
            commands::run_generated_commands(&commands)
        }
        Command::Run => {
            let commands = build_project(program_data, cache, false)
                .with_context(|| "Failed to build project")?;

            commands::run_generated_commands(&commands)?;

            autorun_generated_binary(
                &program_data.compiler.cpp_compiler,
                program_data.build.output_dir,
                program_data.executable.executable_name,
            )
        }
        Command::Test => {
            let commands = build_project(program_data, cache, true)
                .with_context(|| "Failed to build project")?;

            commands::run_generated_commands(&commands)?;

            autorun_generated_binary(
                &program_data.compiler.cpp_compiler,
                program_data.build.output_dir,
                &program_data.tests.test_executable_name,
            )
        }
        Command::New {
            ref name,
            git,
            compiler,
        } => create_templated_project(Path::new("."), &name, git, compiler.into())
            .with_context(|| "Failed to create new project"),
        Command::Cache => todo!(),
    }
}

/// Creates the directory for output the elements generated
/// during the build process. Also, it will generate the
/// ['output_build_dir'/zork], which is a subfolder
/// where Zork dumps the things that needs to work correctly
/// under different conditions.
///
/// Under /zork, some new folders are created:
/// - a /intrinsics folder in created as well,
/// where different specific details of Zork++ are stored
/// related with the C++ compilers
///
/// - a /cache folder, where lives the metadata cached by Zork++
/// in order to track different aspects of the program (last time
/// modified files, last process build time...)
fn create_output_directory(base_path: &Path, model: &ZorkModel) -> Result<()> {
    let out_dir = &model.build.output_dir;
    let compiler = &model.compiler.cpp_compiler;

    // Recursively create a directory and all of its parent components if they are missing
    let modules_path = Path::new(base_path)
        .join(out_dir)
        .join(compiler.to_string())
        .join("modules");
    let zork_path = base_path.join(out_dir).join("zork");
    let zork_cache_path = zork_path.join("cache");
    let zork_intrinsics_path = zork_path.join("intrinsics");

    utils::fs::create_directory(&modules_path.join("interfaces"))?;
    utils::fs::create_directory(&modules_path.join("implementations"))?;
    utils::fs::create_directory(&zork_cache_path)?;
    utils::fs::create_directory(&zork_intrinsics_path)?;

    // TODO This possibly would be temporary
    if compiler.eq(&CppCompiler::CLANG) && cfg!(target_os = "windows") {
        utils::fs::create_file(
            &zork_intrinsics_path,
            "std.h",
            utils::template::resources::STD_HEADER.as_bytes(),
        )?;
        utils::fs::create_file(
            &zork_intrinsics_path,
            "zork.modulemap",
            utils::template::resources::ZORK_MODULEMAP.as_bytes(),
        )?;
    }

    Ok(())
}
