use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use super::commands::arguments::clang_args;
use super::commands::arguments::Argument;
use super::commands::arguments::Arguments;
use crate::cache::CompilersMetadata;
use crate::{
    cache::EnvVars,
    project_model::{compiler::CppCompiler, ZorkModel},
};

/// Convenient datastructure to hold the common args for all the [`super::commands::command_lines::SourceCommandLine`]
/// once they are initialized and stored on the cache, so we just move them once (into this type)
/// and we can pass the struct around to the executors
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct FlyweightData<'a> {
    pub general_args: Arguments<'a>,
    pub shared_args: Arguments<'a>,
    pub std_references: Arguments<'a>, // the correct format of explicitly add the std modular libs
    // to the compiler
    pub compile_but_dont_link: [Argument<'a>; 1],
    pub env_vars: EnvVars,
}

impl<'a> FlyweightData<'a> {
    pub fn new(program_data: &'a ZorkModel, compilers_metadata: &CompilersMetadata) -> Self {
        let mut general_args = Arguments::default();
        general_args.push(program_data.compiler.language_level_arg());
        general_args.extend_from_slice(&program_data.compiler.extra_args);

        let (shared_args, std_references, env_vars) = match program_data.compiler.cpp_compiler {
            CppCompiler::CLANG => {
                let shared_args = generate_clang_flyweight_args(program_data, compilers_metadata);
                (
                    shared_args.0,
                    shared_args.1,
                    &compilers_metadata.msvc.env_vars,
                )
            }
            CppCompiler::MSVC => {
                let shared_args = generate_msvc_flyweight_args(program_data, compilers_metadata);
                (
                    shared_args.0,
                    shared_args.1,
                    &compilers_metadata.msvc.env_vars,
                )
            }
            CppCompiler::GCC => {
                let shared_args = generate_gcc_flyweight_args();
                (
                    shared_args.0,
                    shared_args.1,
                    &compilers_metadata.msvc.env_vars,
                )
            }
        };

        let compile_but_dont_link: [Argument; 1] =
            [Argument::from(match program_data.compiler.cpp_compiler {
                CppCompiler::CLANG | CppCompiler::GCC => "-c",
                CppCompiler::MSVC => "/c",
            })];

        Self {
            general_args,
            shared_args,
            std_references,
            compile_but_dont_link,
            env_vars: env_vars.clone(),
        }
    }
}

fn generate_msvc_flyweight_args<'a>(
    program_data: &ZorkModel<'_>,
    compilers_metadata: &CompilersMetadata<'_>,
) -> SharedArgsStdRefsTuple<'a> {
    let out_dir: &Path = program_data.build.output_dir.as_ref();
    let mut compiler_flyweight_args = Arguments::default();
    let mut std_references = Arguments::default();

    compiler_flyweight_args.push("/EHsc"); // exception_handling_model
    compiler_flyweight_args.push("/nologo");

    compiler_flyweight_args.push("/ifcSearchDir");
    compiler_flyweight_args.push(
        out_dir
            .join(program_data.compiler.cpp_compiler.as_ref())
            .join("modules")
            .join("interfaces"),
    );

    std_references.push("/reference");
    std_references.push(compilers_metadata.msvc.stdlib_bmi_path.clone());
    std_references.push("/reference");
    std_references.push(compilers_metadata.msvc.ccompat_stdlib_bmi_path.clone());

    (compiler_flyweight_args, std_references)
}

type SharedArgsStdRefsTuple<'a> = (Arguments<'a>, Arguments<'a>);

fn generate_clang_flyweight_args<'a>(
    program_data: &'a ZorkModel<'_>,
    compilers_metadata: &CompilersMetadata<'_>,
) -> SharedArgsStdRefsTuple<'a> {
    let mut compiler_flyweight_args = Arguments::default();
    let mut std_references = Arguments::default();

    let out_dir: &Path = program_data.build.output_dir.as_ref();
    let clang_metadata = &compilers_metadata.clang;
    let major = clang_metadata.major;

    compiler_flyweight_args.push_opt(program_data.compiler.stdlib_arg());
    compiler_flyweight_args.push(clang_args::add_prebuilt_module_path(out_dir));

    if major <= 17 {
        compiler_flyweight_args.push("-fimplicit-modules");
        compiler_flyweight_args.push(clang_args::implicit_module_map(out_dir));
    } else {
        std_references.push(format!(
            "-fmodule-file=std={}",
            clang_metadata.stdlib_pcm.display()
        ));
        std_references.push(format!(
            "-fmodule-file=std.compat={}",
            clang_metadata.ccompat_pcm.display()
        ));
    }

    (compiler_flyweight_args, std_references)
}

fn generate_gcc_flyweight_args<'a>() -> SharedArgsStdRefsTuple<'a> {
    let mut compiler_flyweight_args = Arguments::default();
    compiler_flyweight_args.push("-fmodules-ts");
    (compiler_flyweight_args, Arguments::default())
}
