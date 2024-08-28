use crate::domain::commands::arguments::{Argument, Arguments};
use crate::domain::flyweight_data::FlyweightData;
use crate::domain::target::{Target, TargetIdentifier};
use crate::domain::translation_unit::{TranslationUnit, TranslationUnitStatus};
use crate::project_model::compiler::CppCompiler;
use crate::utils::constants::error_messages;
use crate::utils::fs;
use color_eyre::eyre::Context;
use color_eyre::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::{Path, PathBuf};

/// Holds the generated command line arguments for a concrete compiler
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Commands<'a> {
    pub flyweight_data: Option<FlyweightData<'a>>,
    pub modules: ModulesCommands<'a>,
    pub targets: IndexMap<TargetIdentifier<'a>, Target<'a>>,
}

impl<'a> Commands<'a> {
    /// Attempts to remove from the fs all the compilation products of any tracked
    /// [`SourceCommandLine`] of the user declared modules and then,
    /// removes its data from the cache
    pub fn clean_user_modules(&mut self) -> Result<()> {
        self.modules.clear()
    }

    /// Attempts to remove from the fs all the compilation products of any tracked
    /// [`SourceCommandLine`] for all the user declared [`Target`](s) and then,
    /// removes its data from the cache
    pub fn clean_targets(&mut self) -> Result<()> {
        self.targets
            .values_mut()
            .try_for_each(|target| {
                target
                    .sources
                    .iter()
                    .try_for_each(|scl| fs::delete_file(Path::new(&scl.byproduct)))
            })
            .with_context(|| error_messages::FAILURE_CLEANING_TARGETS)?;
        self.targets.clear();
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ModulesCommands<'a> {
    pub cpp_stdlib: Option<SourceCommandLine<'a>>,
    pub c_compat_stdlib: Option<SourceCommandLine<'a>>,
    pub system_modules: Vec<SourceCommandLine<'a>>,
    pub interfaces: Vec<SourceCommandLine<'a>>,
    pub implementations: Vec<SourceCommandLine<'a>>,
}

impl<'a> ModulesCommands<'a> {
    /// Deletes from the fs and then the [`SourceCommandLine`] from the [`ZorkCache`] all
    /// the user [`ModuleInteface`] (and its variants) and [`ModuleImplementation`]
    pub fn clear(&mut self) -> Result<()> {
        self.interfaces
            .iter()
            .try_for_each(|mod_cmd| fs::delete_file(Path::new(&mod_cmd.byproduct)))
            .with_context(|| error_messages::FAILURE_CLEANING_MODULE_INTERFACES)?;
        self.interfaces.clear();

        self.implementations
            .iter()
            .try_for_each(|mod_cmd| fs::delete_file(Path::new(&mod_cmd.byproduct)))
            .with_context(|| error_messages::FAILURE_CLEANING_MODULE_IMPLEMENTATIONS)?;
        self.implementations.clear();

        Ok(())
    }
}

/// Type for representing the command line that will be sent to the target compiler, and
/// store its different components
///
/// * directory*: the path where the translation unit lives
/// * filename*: the translation unit declared name on the fs with the extension
/// * args*: member that holds all the cmd arguments that will be passed to the compiler driver
/// * status*: A [`TranslationUnitStatus`] that represents all the different phases that a source command
///     line can have among all the different iterations of the program, changing according to the modifications
///     over the translation unit in the fs and the result of the build execution
///
/// *byproduct*: A [`PathBuf`] like [`Argument`] which hold the physical address on the filesystem
///     where the compiled object file will be dumped after building it
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct SourceCommandLine<'a> {
    pub directory: PathBuf,
    pub filename: String,
    pub args: Arguments<'a>,
    pub status: TranslationUnitStatus,
    pub byproduct: Argument<'a>,
}

impl<'a> SourceCommandLine<'a> {
    pub fn new<T: TranslationUnit<'a>, B: Into<Argument<'a>>>(
        tu: &T,
        args: Arguments<'a>,
        byproduct: B,
    ) -> Self {
        Self {
            directory: PathBuf::from(tu.parent()),
            filename: tu.filename(),
            args,
            status: TranslationUnitStatus::PendingToBuild,
            byproduct: byproduct.into(),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.directory.join(Path::new(&self.filename))
    }

    pub fn filename(&self) -> &String {
        &self.filename
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct LinkerCommandLine<'a> {
    pub target: Argument<'a>,
    pub args: Arguments<'a>,
    pub extra_args: Arguments<'a>,
    pub execution_result: TranslationUnitStatus,
}

impl<'a> LinkerCommandLine<'a> {
    pub fn get_target_output_for(&self, compiler: CppCompiler) -> Vec<Argument> {
        match compiler {
            CppCompiler::CLANG | CppCompiler::GCC => {
                vec![Argument::from("-o"), self.target.clone()]
            }
            CppCompiler::MSVC => vec![self.target.clone()],
        }
    }
}

impl<'a> Commands<'a> {
    /// Returns a [std::iter::Chain] (behind the opaque impl clause return type signature)
    /// which points to all the generated commands for the two variants of the compilers vendors C++ modular
    /// standard libraries implementations (see: [crate::project_model::compiler::StdLibMode])
    /// joined to all the commands generated for every [TranslationUnit] declared by the user for
    /// its project
    pub fn get_all_modules_command_lines(
        &mut self,
    ) -> impl Iterator<Item = &mut SourceCommandLine<'a>> + Debug {
        self.modules
            .cpp_stdlib
            .as_mut_slice()
            .iter_mut()
            .chain(self.modules.c_compat_stdlib.as_mut_slice().iter_mut())
            .chain(self.modules.system_modules.as_mut_slice().iter_mut())
            .chain(self.modules.interfaces.as_mut_slice().iter_mut())
            .chain(self.modules.implementations.as_mut_slice().iter_mut())
    }
}
