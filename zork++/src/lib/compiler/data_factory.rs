//! Stores Flyweight data structures that allow to reduce n-plications of arguments for every
//! translation unit, having shared data without replicating it until the final command line must
//! be generated in order to be stored (in cache) and executed (in the underlying shell)

use std::{borrow::Cow, path::Path, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    cache::ZorkCache,
    cli::output::arguments::{clang_args, Argument, Arguments},
    domain::target::ExtraArgs,
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

    pub fn get_args_slice(&self) -> impl Iterator<Item = &Argument> {
        self.0.as_slice().iter()
    }

    pub fn get_args_slice_rced(&self) -> impl Iterator<Item = Rc<&Argument>> {
        self.0.as_slice().iter().map(Rc::new)
    }
}

impl<'a> From<&'a ZorkModel<'_>> for CommonArgs {
    fn from(model: &'a ZorkModel<'_>) -> Self {
        let mut common_args = Arguments::default();
        // TODO:
        // Aren't the common args in the end compiler specific ones?
        // Should we remove this DS?
        // Ah no. Maybe we can use it for hold things like -o (shared among all three (MSVC also
        // accepts - in some args instead of /))
        common_args.push(model.compiler.language_level_arg());
        common_args.extend_from_slice(model.compiler.extra_args());

        Self(common_args)
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
    cache: &ZorkCache<'_>,
) -> Box<dyn CompilerCommonArguments> {
    // TODO: consider having a union (enum) instead of a fat ptr, so we can serialize the data
    // and introduce a lifetime on the Argument type to use Cow instead of String
    match model.compiler.cpp_compiler {
        CppCompiler::CLANG => Box::new(ClangCommonArgs::new(model)),
        CppCompiler::MSVC => Box::new(MsvcCommonArgs::new(model, cache)),
        CppCompiler::GCC => Box::new(GccCommonArgs::new()),
    }
}

// TODO: the specific ones, like the object file... can we just create a prototype
// function

/// Allows to have a common interface for any type that represents a data structure which its
/// purpose is to hold common [`Argument`] across the diferent kind of [`TranslationUnit`]
#[typetag::serde(tag = "type")]
pub trait CompilerCommonArguments: std::fmt::Debug {
    fn get_args(&self) -> Arguments;
}
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
impl CompilerCommonArguments for ClangCommonArgs {
    fn get_args(&self) -> Arguments {
        let mut args = Arguments::default();
        args.push(self.std_lib.as_arg());
        args.create_and_push(&self.implicit_modules);
        args.create_and_push(&self.implicit_module_map);
        args.create_and_push(&self.prebuilt_module_path);
        args
    }
}

#[typetag::serde]
impl CompilerCommonArguments for MsvcCommonArgs {
    fn get_args(&self) -> Arguments {
        let mut args = Arguments::default();
        args.create_and_push(&self.exception_handling_model);
        args.create_and_push(&self.no_logo);
        args.create_and_push(&self.ifc_search_dir);
        args.create_and_push(&*self.ifc_search_dir_value);

        args.create_and_push("/reference");
        args.create_and_push(format! {
            "std={}", self.stdlib_ref_path.display()
        });
        args.create_and_push("/reference");
        args.create_and_push(format! {
            "std.compat={}", self.c_compat_stdlib_ref_path.display()
        });
        args
    }
}

#[typetag::serde]
impl CompilerCommonArguments for GccCommonArgs {
    fn get_args(&self) -> Arguments {
        let mut args = Arguments::default();
        args.create_and_push("-fmodules-ts");
        args
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ClangCommonArgs {
    std_lib: StdLib,
    implicit_modules: Cow<'static, str>,
    implicit_module_map: Cow<'static, str>,
    prebuilt_module_path: String,
}
impl ClangCommonArgs {
    pub fn new(model: &ZorkModel<'_>) -> Self {
        let compiler = model.compiler.cpp_compiler;
        let out_dir: &Path = model.build.output_dir.as_ref();

        Self {
            std_lib: model.compiler.std_lib.unwrap_or_default(),
            implicit_modules: "-fimplicit-modules".into(),
            implicit_module_map: clang_args::implicit_module_map(out_dir),
            prebuilt_module_path: clang_args::add_prebuilt_module_path(compiler, out_dir),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct MsvcCommonArgs {
    exception_handling_model: Cow<'static, str>,
    no_logo: Cow<'static, str>,
    reference: Cow<'static, str>,
    ifc_search_dir: Cow<'static, str>,
    ifc_search_dir_value: Cow<'static, Path>,
    stdlib_ref_path: Cow<'static, Path>,
    c_compat_stdlib_ref_path: Cow<'static, Path>,
}
impl MsvcCommonArgs {
    pub fn new(model: &ZorkModel<'_>, cache: &ZorkCache<'_>) -> Self {
        let out_dir: &Path = model.build.output_dir.as_ref();

        Self {
            exception_handling_model: Cow::Borrowed("/EHsc"),
            no_logo: Cow::Borrowed("/nologo"),
            reference: Cow::Borrowed("/reference"),

            ifc_search_dir: Cow::Borrowed("/ifcSearchDir"),
            ifc_search_dir_value: Cow::Owned(
                out_dir
                    .join(model.compiler.cpp_compiler.as_ref())
                    .join("modules")
                    .join("interfaces"),
            ),
            stdlib_ref_path: Cow::Owned(cache.compilers_metadata.msvc.stdlib_bmi_path.clone()),
            c_compat_stdlib_ref_path: Cow::Owned(
                cache.compilers_metadata.msvc.c_stdlib_bmi_path.clone(),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GccCommonArgs {
    modules_ts: Cow<'static, str>,
}
impl GccCommonArgs {
    pub fn new() -> Self {
        Self {
            modules_ts: Cow::Borrowed("-fmodules-ts"),
        }
    }
}
