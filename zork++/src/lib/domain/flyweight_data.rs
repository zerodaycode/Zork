use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::Result;

use super::commands::arguments::Arguments;
use crate::cache::CompilersMetadata;
use crate::compiler::data_factory::CommonArgs;
use crate::compiler::data_factory::CompilerCommonArguments;
use crate::{
    cache::EnvVars,
    project_model::{compiler::CppCompiler, ZorkModel},
    utils::constants::error_messages,
};

/// Convenient datastructure to hold the common args for all the [`super::commands::command_lines::SourceCommandLine`]
/// once they are initialized and stored on the cache, so we just move them once (into this type)
/// and we can pass the struct around to the executors
pub struct FlyweightData<'a> {
    pub general_args: Arguments<'a>,
    pub shared_args: Arguments<'a>,
    pub env_vars: &'a EnvVars,
}

impl<'a> FlyweightData<'a> {
    pub fn new(
        program_data: &ZorkModel,
        general_args: &'a mut Option<CommonArgs<'_>>,
        compiler_common_args: &'a mut Option<Box<dyn CompilerCommonArguments>>,
        compilers_metadata: &'a CompilersMetadata,
    ) -> Result<Self> {
        let general_args = general_args
            .as_mut()
            .with_context(|| error_messages::GENERAL_ARGS_NOT_FOUND)?
            .get_args();

        let shared_args = compiler_common_args
            .as_mut()
            .with_context(|| error_messages::COMPILER_SPECIFIC_COMMON_ARGS_NOT_FOUND)?
            .get_args();

        let env_vars = match program_data.compiler.cpp_compiler {
            CppCompiler::MSVC => &compilers_metadata.msvc.env_vars,
            CppCompiler::CLANG => &compilers_metadata.clang.env_vars,
            CppCompiler::GCC => &compilers_metadata.gcc.env_vars,
        };

        Ok(Self {
            general_args,
            shared_args,
            env_vars,
        })
    }
}
