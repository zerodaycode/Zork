use crate::compiler::data_factory::{CommonArgs, CompilerCommonArguments};
use crate::domain::commands::arguments::{Argument, Arguments};
use crate::domain::target::{Target, TargetIdentifier};
use crate::domain::translation_unit::{TranslationUnit, TranslationUnitStatus};
use crate::project_model::compiler::CppCompiler;
use crate::utils::constants::error_messages;
use color_eyre::eyre::{ContextCompat, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::{Path, PathBuf};

/// Type for representing the command line that will be sent to the target compiler, and
/// store its different components
///
/// * directory*: the path where the translation unit lives
/// * filename*: the translation unit declared name on the fs with the extension
/// * args*: member that holds all the cmd arguments that will be passed to the compiler driver
/// * status*: A [`TranslationUnitStatus`] that represents all the different phases that a source command
/// line can have among all the different iterations of the program, changing according to the modifications
/// over the translation unit in the fs and the result of the build execution
/// *byproduct*: A [`PathBuf`] like [`Argument`] which hold the physical address on the filesystem
/// where the compiled object file will be dumped after building it
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
    // TODO: review if we do need this struct yet, since target does
    // the same
    link_modules: bool, // TODO: pending
    pub target: Argument<'a>,
    pub modules_byproducts: Arguments<'a>,
    pub byproducts: Arguments<'a>,
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

    /// Saves the path at which a compilation product of any translation unit will be placed,
    /// in order to add it to the files that will be linked to generate the final product
    /// in the two-phase compilation model
    pub fn add_byproduct_path(&mut self, path: PathBuf) {
        self.byproducts.push(path);
    }
}

/// Holds the generated command line arguments for a concrete compiler
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Commands<'a> {
    pub general_args: Option<CommonArgs<'a>>,
    pub compiler_common_args: Option<Box<dyn CompilerCommonArguments>>,
    pub modules: ModulesCommands<'a>,
    pub targets: IndexMap<TargetIdentifier<'a>, Target<'a>>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ModulesCommands<'a> {
    pub cpp_stdlib: Option<SourceCommandLine<'a>>,
    pub c_compat_stdlib: Option<SourceCommandLine<'a>>,
    pub system_modules: Vec<SourceCommandLine<'a>>,
    pub interfaces: Vec<SourceCommandLine<'a>>,
    pub implementations: Vec<SourceCommandLine<'a>>,
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

    // TODO: unused, think that doesn't makes sense anymore with the current architechture
    pub fn add_linker_file_path(
        &mut self,
        target_identifier: &TargetIdentifier<'a>,
        path: PathBuf,
    ) -> Result<()> {
        self.targets
            .get_mut(target_identifier)
            .with_context(|| {
                format!(
                    "{}: {:?}",
                    error_messages::TARGET_ENTRY_NOT_FOUND,
                    target_identifier
                )
            })?
            .linker
            .add_byproduct_path(path);
        Ok(())
    }
}
