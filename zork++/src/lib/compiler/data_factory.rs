//! Stores Flyweight data structures that allow to reduce n-plications of arguments for every
//! translation unit, having shared data without replicating it until the final command line must
//! be generated in order to be stored (in cache) and executed (in the underlying shell)

use std::{borrow::Cow, path::Path, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    bounds::ExtraArgs,
    cli::output::arguments::{clang_args, Argument, Arguments},
    project_model::compiler::{CppCompiler, StdLib},
    project_model::ZorkModel,
};

/// Holds the common arguments across all the different command lines regarding the target compiler
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CommonArgs(Arguments);
impl CommonArgs {
    pub fn get_args(&self) -> Arguments {
        self.0.clone()
    }

    // pub fn get_args_slice(&self) -> &[Rc<Argument>] {
    pub fn get_args_slice(&self) -> impl Iterator<Item = Rc<&Argument>> {
        self.0.as_slice().iter().map(|arg| Rc::new(arg))
    }
}

impl<'a> From<&'a ZorkModel<'_>> for CommonArgs {
    fn from(model: &'a ZorkModel<'_>) -> Self {
        let mut common_args = Arguments::default();
        common_args.push(model.compiler.language_level_arg());
        common_args.extend_from_slice(model.compiler.extra_args());

        Self (common_args)
    }
}

impl IntoIterator for CommonArgs {
    type Item = Argument;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Factory function for bring the data structure that holds the common arguments of a source
/// command line for every translation unit, regardeless the underlying choosen compiler
pub fn compiler_common_arguments_factory(
    model: &ZorkModel<'_>,
) -> Box<dyn CompilerCommonArguments> {
    // TODO: consider having a union (enum) instead of a fat ptr
    match model.compiler.cpp_compiler {
        CppCompiler::CLANG => Box::new(ClangCommonArgs::new(model)),
        CppCompiler::MSVC => Box::new(MsvcCommonArgs::new()),
        CppCompiler::GCC => Box::new(GccCommonArgs::new()),
    }
}

// TODO: the specific ones, like the object file... can we just create a prototype
// function

/// Allows to have a common interface for any type that represents a data structure which its
/// purpose is to hold common [`Argument`] across the diferent kind of [`TranslationUnit`]
#[typetag::serde(tag = "type")]
pub trait CompilerCommonArguments {}
impl Default for Box<dyn CompilerCommonArguments> {
    fn default() -> Self {
        Box::<ClangCommonArgs>::default() // TODO: isn't this a code smell?
                                          // TODO: should we just panic? Or maybe fix the default? Or maybe have an associated
                                          // and pass the compiler to the trait fn? So we can ensure that the default has sense?
                                          // TODO: we can just fix as well the serialization function, removing the default
    }
}

/// TODO: the typetag library doesn't support yet the deserialization of generic impls, only
/// serialization, so there's no point on having any primites
#[typetag::serde]
impl CompilerCommonArguments for ClangCommonArgs {}
#[typetag::serde]
impl CompilerCommonArguments for MsvcCommonArgs {}
#[typetag::serde]
impl CompilerCommonArguments for GccCommonArgs {}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ClangCommonArgs {
    // TODO: a HashMap per kind of translation unit which stores the common ones
    // for every different kind of translation unit
    std_lib: StdLib, // TODO: technically, should this already be an arg? or should we decouple the
    // project model for the Argument(s) type(s)?
    implicit_modules: Cow<'static, str>,
    implicit_module_map: Argument,
    prebuilt_module_path: Cow<'static, str>,
}
impl ClangCommonArgs {
    pub fn new(model: &ZorkModel<'_>) -> Self {
        let out_dir: &Path = model.build.output_dir.as_ref();

        Self {
            std_lib: model.compiler.std_lib.unwrap_or_default(),
            implicit_modules: "-fimplicit-modules".into(),
            implicit_module_map: clang_args::implicit_module_map(out_dir),
            prebuilt_module_path: Cow::Owned(format!(
                "-fprebuilt-module-path={}/clang/modules/interfaces",
                out_dir.display()
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct MsvcCommonArgs {
    exception_handling_model: Cow<'static, str>,
    /* no_logo: &'a str,
    no_compile: &'a str, // TODO: should be in the general and pass in the model? */
    // ref_stdlib: &'static str, // TODO: this are tecnically two args, /reference and the value
    // ref_stdlib_compat: &'static str, // TODO: this are tecnically two args, /reference and the value
    // TODO: split the dual cases per switches
    // TODO: can we have switches like tuples? like switch-value pairs?
}
impl MsvcCommonArgs {
    pub fn new() -> Self {
        Self {
            exception_handling_model: Cow::Borrowed("/EHsc"),
            /* no_logo: "nologo",
            no_compile: "/c", */
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GccCommonArgs {}
impl GccCommonArgs {
    pub fn new() -> Self {
        Self {}
    }
}
