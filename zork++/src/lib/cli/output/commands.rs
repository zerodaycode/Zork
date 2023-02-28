use std::{path::{Path, PathBuf}, process::{ExitCode, ExitStatus}, os::windows::process::ExitStatusExt};

///! Contains helpers and data structure to process in
/// a nice and neat way the commands generated to be executed
/// by Zork++
use crate::{cache::ZorkCache, project_model::compiler::CppCompiler, utils::constants};
use color_eyre::{eyre::Context, Result, Report};

use super::arguments::Argument;


pub fn run_generated_commands(commands: &mut Commands<'_>, cache: &ZorkCache) -> Result<ExitStatus> {
    if !commands.interfaces.is_empty() {
        log::debug!("Executing the commands for the module interfaces...");
    }
    for miu in &mut commands.interfaces {
        if !miu.processed {
            let res = execute_command(&commands.compiler, &miu.args, cache);
            match res {
                Ok(r) => {
                    miu.execution_result = Ok(r);
                    if !r.success() { return Ok(r); }
                },
                Err(e) => return Err(e),
            }
        } else {
            log::debug!("Translation unit: {:?} was not modified since the last iteration. No need to rebuilt it again", miu.path);
        }
    }

    if !commands.implementations.is_empty() {
        log::debug!("Executing the commands for the module implementations...");
    }
    for implm in &mut commands.implementations {
        if !implm.processed {
            let res = execute_command(&commands.compiler, &implm.args, cache);
            match res {
                Ok(r) => {
                    implm.execution_result = Ok(r);
                    if !r.success() { return Ok(r); }
                },
                Err(e) => return Err(e),
            }
        } else {
            log::debug!("Translation unit: {:?} was not modified since the last iteration. No need to rebuilt it again", implm.path);
        }
    }

    if !commands.sources.is_empty() {
        log::debug!("Executing the main command line...");
        execute_command(&commands.compiler, &commands.sources, cache)?;
    }

    Ok(ExitStatus::from_raw(0))
}

/// Executes a new [`std::process::Command`] to run the generated binary
/// after the build process in the specified shell
pub fn autorun_generated_binary(
    compiler: &CppCompiler,
    output_dir: &Path,
    executable_name: &str,
) -> Result<()> {
    let args = &[Argument::from(
        output_dir
            .join(compiler.as_ref())
            .join(executable_name)
            .with_extension(constants::BINARY_EXTENSION),
    )];

    log::info!(
        "[{compiler}] - Executing the generated binary => {:?}",
        args.join(" ")
    );

    std::process::Command::new(Argument::from(
        output_dir.join(compiler.as_ref()).join(executable_name),
    ))
    .spawn()?
    .wait()
    .with_context(|| format!("[{compiler}] - Command {:?} failed!", args.join(" ")))?;

    Ok(())
}

/// Executes a new [`std::process::Command`] configured according the choosen
/// compiler and the current operating system
fn execute_command(
    compiler: &CppCompiler,
    arguments: &[Argument<'_>],
    cache: &ZorkCache,
) -> Result<ExitStatus, Report> {
    log::debug!(
        "[{compiler}] - Executing command => {:?}",
        format!("{} {}", compiler.get_driver(), arguments.join(" "))
    );

    let process = if compiler.eq(&CppCompiler::MSVC) {
        std::process::Command::new(
            cache
                .compilers_metadata
                .msvc
                .dev_commands_prompt
                .as_ref()
                .expect("Zork++ wasn't able to found a correct installation of MSVC"),
        )
        .arg("&&")
        .arg(compiler.get_driver())
        .args(arguments)
        .spawn()?
        .wait()
        .with_context(|| format!("[{compiler}] - Command {:?} failed!", arguments.join(" ")))
    } else {
        std::process::Command::new(compiler.get_driver())
            .args(arguments)
            .spawn()?
            .wait()
            .with_context(|| format!("[{compiler}] - Command {:?} failed!", arguments.join(" ")))
    };

    log::debug!("[{compiler}] - Result: {:?}\n", process);
    process
}

/// Executes a new [`std::process::Command`] configured according the choosen
/// compiler and the current operating system composed of multiple prebuilt command
/// lines joining them in one statement
///
/// TODO! Probably it would be better only make a big command formed by all the commands
/// for the MSVC compiler in order to avoid to launch the developers command prompt
/// for every commmand, but, as observed, generally speaking opening a shell under
/// Unix using Clang or GCC it's extremily fast, so we can mantain the curren architecture
/// of opening a shell for command, so the user is able to track better failed commands
fn _execute_commands(
    compiler: &CppCompiler,
    arguments_for_commands: &[Vec<Argument<'_>>],
    cache: &ZorkCache,
) -> Result<()> {
    let mut commands = if compiler.eq(&CppCompiler::MSVC) {
        std::process::Command::new(
            cache
                .compilers_metadata
                .msvc
                .dev_commands_prompt
                .as_ref()
                .expect("Zork++ wasn't able to found a correct installation of MSVC"),
        )
    } else {
        std::process::Command::new("sh")
    };

    arguments_for_commands.iter().for_each(|args_collection| {
        log::debug!(
            "[{compiler}] - Generating command => {:?}",
            format!("{} {}", compiler.get_driver(), args_collection.join(" "))
        );

        commands
            .arg("&&")
            .arg(compiler.get_driver())
            .args(args_collection);
    });

    commands
        .spawn()?
        .wait()
        .with_context(|| format!("[{compiler}] - Command {commands:?} failed!"))?;

    log::info!("[{compiler}] - Result: {:?}", commands);
    Ok(())
}

/// Holds a collection of heap allocated arguments. This is introduced in the
/// v0.7.0, for just wrapping the vector that holds the arguments, and for hold
/// a flag that will indicate us that this command line will be used in a module,
/// and that module was already built, and the module source file didn't change
/// since the last iteration of Zork++
#[derive(Debug)]
pub struct ModuleCommandLine<'a> {
    pub path: PathBuf,
    pub args: Vec<Argument<'a>>,
    pub processed: bool,
    pub execution_result: Result<ExitStatus, Report>
}

impl<'a> From<Vec<Argument<'a>>> for ModuleCommandLine<'a> {
    fn from(value: Vec<Argument<'a>>) -> Self {
        Self {
            path: PathBuf::new(),
            args: value,
            processed: false,
            execution_result: Ok(ExitStatus::from_raw(0))
        }
    }
}

impl<'a> IntoIterator for ModuleCommandLine<'a> {
    type Item = Argument<'a>;

    type IntoIter = std::vec::IntoIter<Argument<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
    }
}

impl<'a> FromIterator<Argument<'a>> for ModuleCommandLine<'a> {
    fn from_iter<T: IntoIterator<Item = Argument<'a>>>(iter: T) -> Self {
        let mut m = ModuleCommandLine {
            path: PathBuf::new(),
            args: Vec::new(),
            processed: false,
            execution_result: Ok(ExitStatus::from_raw(0))
        };

        for arg in iter {
            m.args.push(arg)
        }

        m
    }
}

/// Holds the generated command line arguments for a concrete compiler
#[derive(Debug)]
pub struct Commands<'a> {
    pub compiler: CppCompiler,
    pub system_modules: Vec<Vec<Argument<'a>>>,
    pub interfaces: Vec<ModuleCommandLine<'a>>,
    pub implementations: Vec<ModuleCommandLine<'a>>,
    pub sources: Vec<Argument<'a>>,
    pub generated_files_paths: Vec<Argument<'a>>,
}

impl<'a> Commands<'a> {
    pub fn new(compiler: &'a CppCompiler) -> Self {
        Self {
            compiler: *compiler,
            system_modules: Vec::with_capacity(0),
            interfaces: Vec::with_capacity(0),
            implementations: Vec::with_capacity(0),
            sources: Vec::with_capacity(0),
            generated_files_paths: Vec::with_capacity(0),
        }
    }
}

impl<'a> core::fmt::Display for Commands<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Commands for [{}]:\n- Interfaces: {:?},\n- Implementations: {:?},\n- Main command line: {:?}",
            self.compiler,
            self.interfaces.iter().map(|vec| { vec.args.iter().map(|e| e.value).collect::<Vec<_>>().join(" "); }),
            self.implementations.iter().map(|vec| { vec.args.iter().map(|e| e.value).collect::<Vec<_>>().join(" "); }),
            self.sources.iter().map(|e| e.value).collect::<Vec<_>>().join(" ")
        )
    }
}
