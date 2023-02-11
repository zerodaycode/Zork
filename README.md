<p align="center">
  <a href="" rel="noopener">
 <img width=300px height=200px style="border-radius: 50%" src="./assets/zork++_logo.png" alt="Zork++ Logo"></a>
</p>

<h1 align="center">The Zork++ project</h1>

<h3 align="center"> A modern C++ project manager and build system for modern C++
</h3>
</br>

<div align="center">

[![Code Quality](https://github.com/zerodaycode/Zork/actions/workflows/pylint.yml/badge.svg?branch=main)](https://github.com/zerodaycode/Zork/actions/workflows/code-quality.yml)
[![GitHub Issues](https://img.shields.io/github/issues/zerodaycode/Zork.svg)](https://github.com/zerodaycode/Zork/issues) </br>
[![GitHub Pull Requests](https://img.shields.io/github/issues-pr/zerodaycode/Zork.svg)](https://github.com/zerodaycode/Zork/pulls)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](/LICENSE)
[![Windows Installer](https://github.com/zerodaycode/Zork/actions/workflows/release.yml/badge.svg?branch=main)](https://github.com/zerodaycode/Zork/actions/workflows/release.yml)
![Coverage](https://zerodaycode.github.io/Zork/coverage-status.svg)
</div>

---
</br>

# 📝 Table of Contents

- [About](#about)
- [Getting Started](#getting_started)
- [The `zork.toml` quick start](#usage)
- [The `zork.toml` reference guide](#zork_toml_reference)
- [The `Zork++` command_line](#zork_command_line)
- [C++23 `import std;`](#import_std)
- [The developers and contributors guide](#devs_guide)
- [TODO ZONE](#todo_zone)
- [Built Using](#built_using)
- [Contributing](../CONTRIBUTING.md)
- [License](./LICENSE)
- [Authors](#authors)
- [Acknowledgments](#acknowledgement)

# 🧐 About <a name = "about"></a>

`Zork++` was born out of the need to build `C++` projects with the *modules* feature, introduced in the `C++20` standard.

Due to the existing limitations in the other build systems and the difficulty of using the `C++20` modules in a consistent manner in the first years after being released as a language feature,
we started to design a tool that, just takes some `C++` files, generates the necessary command lines, invokes the compiler and produces the desired executable/library!

# 🏁 Getting Started <a name = "getting_started"></a>

## - Installing

The asiest way to start with `Zork++` is to download the [latest release
available](https://github.com/zerodaycode/Zork/releases) for your current operating system.

In Windows:

- We provide you an automated installer that will install the `Zork++` binary in your system and will put the program in PATH automatically

In Linux:

- You will have a `tar.gz` compressed file or a `.deb` package to choose. Extract it an then, place the binary somewhere in the system where makes sense and it's confortable for you. Typically, directories like `/usr/bin/local` or `/opt/...` are good places.
Remeber to add the PATH to the binary to your system PATH permanently, so you can access the program directly from the command line.

- In MacOS or another targets:

- We don't provide installers or precompiled binaries for another operating systems. But, as far as your target is some of the supported by `Rust`, you could download the source code, and build it from scratch, generating a new binary that you can use. You can checkout [the list of available targets here](https://doc.rust-lang.org/nightly/rustc/platform-support.html)

## -  Prerequisites

In order to work with `Zork++`, you will need a `C++` toolchain and a compiler in your system.
Currently, we are supporting the major three compilers in the ecosystem:

- LLVM's Clang
- GNU's GCC
- Microsoft's MSVC (only supported on Windows)

## - Generating a new C++ project

You can use `Zork++` with an already existing project, or start a new one from a terminal.

- Choose an empty folder to kick off with a fresh start.
- Assuming that you have `Zork++` in PATH, type:

`$ zork++ new github_example --compiler clang`

After a moment, a new project should be created in your desired directory.
And after that, move inside the folder, and type:

`$ zork++ run`

An output similar to the one below should appear in your terminal

```stdout
[2023-02-08T20:03:25Z INFO Zork++] Launching a new Zork++ program
[2023-02-08T20:03:25Z INFO Zork++] [SUCCESS] - The process ended succesfully, taking a total time in complete of: 349 ms

Hello from an autogenerated Zork project!
The program is running with: Clang

RESULT '+': 10
RESULT '-': 6
RESULT '*': 16
RESULT '/': 1
```

What happened here?

- We are calling the `Zork++` executable previouslly installed on the system
- With the new argument, we are instructing `Zork++` to autogenerate a new `C++` based on *modules* project, choosing `Clang` as compiler. The project contains a predeterminated structure as shown below
- With the `run` command, we are building the project based on the configuration that is written in the `zork_clang.toml` configuration file, and after build, Zork++ will automatically run the generated binary

<p align="center">
  <a href="" rel="noopener">
 <img width=250px height=300px src="./assets/autogenerated_project_structure.png" alt="Zork++ autogenerated example"></a>
</p>

*An autogenerated project using the `Zork++` command line*

### An overview of the autogenerated project structure

- `dependencies` => Empty folder. We recommend you to put your third-party or external libraries, headers, etc. , in this folder.
- `ifc` => stands for *interfaces*. We put here the *module interface units* of the project.
- `src` => where the *module implementation files* live.
- `tests` => where the *unit tests* and the *integration tests* of your new project are.
- `main.cpp` => The classical entry point for a `cpp` program.
- `zork_clang.toml` => Finally! We arrived at the most important part that you will need to understand when working with `Zork++`.
See the [The zork.toml config file](#zork_conf) section to have a better understanding on how to write the configuration file and your project.

> Note that this structure is just a guideline. You may prefer to organize your files in a complete different way. We are just providing a predefined standard to quickly start working with it.

## Let's explore a little bit the `out` directory

Here is where your compiler will place all the elements after the compilation and linkage process. It may contain a binary, a library... Also, contains the intermediate files needed to create the final executable or library, and some special folders of `Zork++`, as the cache and the intrinsics. Don't worry aboout them, they aren't relevant for the moment.

Also, a compiler specfic directory is created per every different compiler invoked in the configuration files, so everything build process for every different compiler is splitted in it's own folder.

Let's see the folders:

- `modules` => it's a generated folder by `Zork` where the compiler will place all the precompiled module interfaces, the object files for the `cpp` source files...
- `zork/cache` => We dump here the cache files that stores useful data to speed up the process, or to track some relevant data for us, or for store the generated commands for the user, if you want to inspect them.
- `zork/intrinsics` => this is a special one. Sometimes `Zork` needs additional things to work properly. So this is the place where those neccesary things live. See [Windows special requeriments](#windows_special_requeriments) for more info.
- `github_example.exe` => This is the final binary generated for the project.

# 🔧 The Zork's `zork.toml` config file <a name="usage"></a>

Wow! We finally arrived at the most important part of `Zork++`. The `zork.toml` configuration file is the core of the project. This is where you define the set-up of your project, instructing `Zork++` to behave as you expect. We are going to take the `zork.toml` from the example of the last entry

Here is the configuration file for the example above:

```toml
#This file it's autogenerated as an example of a Zork config file
[project]
name = "github-example"
authors = [ "Zero Day Code" ]  # Replace this for the real authors

[compiler]
cpp_compiler = "clang"
cpp_standard = "20"
std_lib = "libc++"  # This concrete property will only be applied to Clang

[build]
output_dir = "./out"

[executable]
executable_name = "github-example"
sources = [
    "./github-example/*.cpp"
]

[tests]
tests_executable_name = "zork_proj_tests"
sources = [
    "./github-example/*.cpp"
]

[modules]
base_ifcs_dir = "./github-example/ifc"
interfaces = [ 
    { file = 'math.cppm' }    
]
base_impls_dir = "./github-example/src"
implementations = [
    { file = 'math.cpp' },
    { file = 'math2.cpp', dependencies = ['math'] }
]
sys_modules = ['iostream']
```

This is `toml` table syntax. You may choose whatever `toml` available syntax you prefer, since is just a regular `toml`.

For a full reference of every property available under a specific section, please see the [zork.toml](#zork_toml_reference) reference guide.

## Let's briefly discuss every section, to get a general perspective of what we are building

- `[project]` => Here you specify the metadata for the project. This could be potentially more relevant in the future, to allow users to autopublish projects in some web sites.

- `[compiler]` => The configuration for the `C++` compiler options. From the desired compiler that you want to use, the desired language level of the C++ ISO standard and the `std` library vendor that you want to link against.
Now, we are only using this last one property con configure `libc++` or `libstdc++` under Unix systems for Clang, so specifiying this option only will take effect under this conditions

- `[build]` => Specifics about how and where the generated files are dumped or treated. Here you specify where you want to create that directory.

- `[executable]` => Whenever `Zork++` sees this attribute, it knows that he must produce an executable. You must specify the name of the generated binary and the sources that will be taken in consideration to produce such executable.

- `[modules]` => The core section to instruct the compiler to work with `C++20 modules`. The most important are the base path to the *interfaces* and *implementation* files (usually you don't want to spread your files elsewhere), so `Zork++` knows where it should look for them, and the `interfaces` and `implementation` files where you speficy exactly what modules are composing your project.

- `[tests]` => Tests attribute allows you to run the tests written for
your application in a convenient way. You only have to provide the sources
and... ready to go! `Zork++` will take care of all the rest.

For now, tests run independent from the executables or library generations. So, if you want to check the health of your applications and run your tests, just invoke Zork's binary and pass to the command line the `test` argument.

`$ zork++ -v test`

You manually must provide a tests suite for now. There are some plans to include one or more of the major ones shipped with `Zork++` directly, like `boost::ut` or `Catch2`, but for now, you must manually download the source files and pass them (if applies) to `Zork++`.

`base_path` property (optional) that allows you to reduce the path needed to specify every time for every source file.

## Additional notes on the `[modules]` attribute

> Whenever you declare a module interface or a module implementation in the configuration file, you must take in consideration that sometimes modules (both interfaces or implementations) depend on other modules. Dependencies of one or more modules are declared as shown below:

```toml
interfaces = [ 
    { file = 'math.cppm' }    
]
implementations = [
    { file = 'math.cpp' }, # Direct mapping with the interface `math`
    { file = 'math2.cpp', dependencies = ['math'] } 
    # math2 isn't the same as math, so we need to specify the `math` dependency.
]
```

> The *module interface units*  can take any number of module dependencies as they wish, but there's no direct mapping on the no dependency declaration, because it's an interface unit.

> On the implementations, if the relation has no declared dependencies,
(meaning that only the file name of the *module implementation*
is declared here), `Zork++` will assume that the unique module interface that the module implementation depends on has the same name of the *file* property declared for the module implementation.

> A *module implementation unit* always depends, on an interface unit at least,
which is, the *module interface* that public declared the module.
That's why `Zork++` assumes that if the implementation unit does not explicitly
declares an interface unit, an interface unit exists with the same name
(without file extension) in the interfaces directory.

> If the user does not use the same file name for both
the interface and the declaration, and not direct dependency is declared, a compiler error will be thrown because `Zork++` does not care about wrong
specified dependencies.

## Module partitions

One thing that we didn't already discussed is the `module partitions`. Described by the standard, the are two kinds of partitions, known as `module interface partitions` and `module implementation partition`, or `internal partition`. Both of them serves to the same purpose, allow to better organize and modularize the code when projects start to go bigger, or need a particular source code layout.

`Zork++` supports `module partitions` for every supported compiler, but they still have some peculiarities. Let's review them with an example: 

```toml
[modules]
interfaces = [ 
    { file = 'interface_partition.cppm', partition = { module = 'partitions' } },
    { file = 'internal_partition.cpp', partition = { module = 'partitions', partition_name = 'internal_partition', is_internal_partition = true } },
    { file = 'partitions.cppm' }
]
``` 
*A closer look on how to work with module partitions within Zork++*

We included insider the `interfaces` key because, most of the time, other module interfaces will require some partition, and having a separate key for them will broke the way of letting you decide in which order the translation units must be processed.

So, whenever you see an *"interface"* that has a *partition* property, you must know that we are "downcasting" a module interface to some kind of partition.

So, pecularities by compiler at the time of writting.

- GCC => NONE! GCC comes with a powerful module cache, and no further threatment is required. You can write module partitions without worry to declared them in `Zork++` even.
- Clang => This requires you to fully specify partitions, indicating the module property, which is the property that tells `Zork++` which is its related primary module interface, and the partition name. If partition name isn't present, we will assume the name of the partition file.
- MSVC => Basically, we take advante of the fantastic `MSVC` implicit module lookup.
This is that you aren't obligated to explicitly declare module names, neither module partition names... but, there's an specific case, `internal module partitions`. So, whenever you have an internal module partition, you must declare your translation unit as `partition`, and then provide at least `module` and `is_internal_partition` in order to make it work

> Note that in future releases, things about module partitions may change drastically (or not!). We are expecting, for example, Clang to implement a good way of making implicit declarations but having the oportunity to specify a concrete output directory, among other things in other compilers too.

## The sys_modules property

`Clang` and `GCC`, requires to precompile first the classical system headers
to be importable as modules whenever you use the `import <iostream>` or another standard library component instead of using include directives.
Every time that you want to use `import<sys_module>` in your projects, you can instruct `Zork` to precompile first them for you
in order to make it available to the compiler.

# 📑 The `zork.toml` reference guide <a name="zork_toml_reference"></a>

## A guide with all the sections, with the attributes and its properties available on Zork, indicating when an attribute or a property is optional or mandatory, and for the properties, if they have default values. This is a simplified Rust code, to show the available specs. Types marked with `Option<T>` are optional values

```Rust
/// The complete hierarchy of keys
ZorkConfigFile {
    project: ProjectAttribute,
    compiler: CompilerAttribute,
    build: Option<BuildAttribute>,
    executable: Option<ExecutableAttribute>,
    modules: Option<ModulesAttribute>,
    tests: Option<TestsAttribute>,
}

/// The [project] key 
ProjectAttribute {
    name: &'a str
    authors: Option<Vec<str>>,
}

/// The [compiler] key 
CompilerAttribute {
    cpp_compiler: CppCompiler, // clang, msvc or gcc
    cpp_standard: LanguageLevel, // but as a string, for ex: '20'
    std_lib: Option<str>, // libc++ or stdlibc++
    extra_args: Option<Vec<str>>
}

/// The [build] key 
BuildAttribute {
    output_dir: Option<str>,
}

/// The [executable] key
ExecutableAttribute {
    executable_name: Option<str>,
    sources_base_path: Option<str>,
    sources: Option<Vec<str>>,
    extra_args: Option<Vec<str>>,
}

/// [`ModulesAttribute`] -  The core section to instruct the compiler to work with C++20 modules. The most important are the base path to the interfaces and implementation files
/// * `base_ifcs_dir`- Base directory to shorcut the path of the implementation files
/// * `interfaces` - A list to define the module interface translation units for the project
/// * `base_impls_dir` - Base directory to shorcut the path of the implementation files
/// * `implementations` - A list to define the module interface translation units for the project
/// * `sys_modules` - An array field explicitly declare which system headers must be precompiled
ModulesAttribute {
    base_ifcs_dir: Option<str>,
    interfaces: Option<Vec<ModuleInterface>>,
    base_impls_dir: Option<str>,
    implementations: Option<Vec<ModuleImplementation>>,
    sys_modules: Option<Vec<str>>,
}

/// The [tests] key
TestsAttribute {
    test_executable_name: Option<str>,
    sources_base_path: Option<str>,
    sources: Option<Vec<str>>,
    extra_args: Option<Vec<str>>,
} 
```

## A closer look on the ModulesAttribute key

```Rust
/// [`ModuleInterface`] -  A module interface structure for dealing
/// with the parse work of prebuild module interface units
///
/// * `file`- The path of a primary module interface 
/// (relative to base_ifcs_path if applies)
///
/// * `module_name` - An optional field for make an explicit
/// declaration of the C++ module on this module interface 
/// with the `export module 'module_name' statement.
/// If this attribute isn't present, Zork++ will assume that the
/// C++ module declared within this file is equals 
/// to the filename
///
/// * `partition` - Whenever this attribute is present, 
/// we are telling Zork++ that the actual translation unit
/// is a partition, either an interface partition 
/// or an implementation partition unit
///
/// * `dependencies` - An optional array field for declare the module interfaces
/// in which this file is dependent on
ModuleInterface {
    file: str,
    module_name: Option<str>,
    partition: Option<ModulePartition>,
    dependencies: Option<Vec<str>>,
}

/// [`ModulePartition`] - Type for dealing with the parse work
/// of module partitions, either interfaces or implementations
///
/// * `module`- The interface module unit that this partitions belongs to
///
/// * `partition_name` - An optional field for explicitly declare the name of a module interface
/// partition, or a module implementation partition.
/// Currently this requirement is only needed if your partitions 
/// file names aren't declared as the modules convention,
/// that is `module_name-partition_name.extension`
///
/// * `is_internal_partition` - Optional field for declare that 
/// the module is actually a module for hold implementation
/// details, known as module implementation partitions.
/// This option only takes effect with MSVC
ModulePartition {
    module: str,
    partition_name: Option<&str>,
    is_internal_partition: Option<bool>,
}

/// [`ModuleImplementation`] -  Type for dealing with the parse work
/// of module implementation translation units
///
/// * `file`- The path of a primary module interface (relative to base_ifcs_path)
/// * `dependencies` - An optional array field for declare the module interfaces
/// in which this file is dependent on
ModuleImplementation<'a> {
    file: str,
    dependencies: Option<Vec<str>>,
}
```

## Some specific configuration values

> Take in consideration some values that must be mandatory written
in a specific way, or they just have a few options available on how
they must be specified in the command line, like:

- The supported CPP standards => '20', '23', '1a', '2a', '1x', '2x' or 'latest'
- The supported compilers:
  - clang => (alias = "CLANG", alias = "Clang", alias = "clang")
  - msvc  => (alias = "MSVC", alias = "Msvc", alias = "msvc")
  - gcc   => (alias = "MSVC", alias = "Msvc", alias = "msvc")
- The supported standard libraries to link against (only applies to `Clang`) => 'stdlibc++' or 'libc++'

# 📑 The `Zork++` command line interface <a name="zork_command_line"></a>

`Zork++` comes with a minimalist yet powerful command line interface.
Our direct intention was to mimic the standard way of work with `Rust's` `Cargo` cli,
as is a world class tool well known and valorated.
To summarize, we are offering the following commands and arguments:

- *build* => just compiles the project
- *run* => compiles the project and then, runs the generated binary
- *test* => compiles the project and then runs the test suite linked to the files described
in the configuration file automatically
- *new* => generates a new `C++20` onwards template project with a minimal configuration and
a minimal setup. This command includes some arguments to make it more flexible, like:
  - *name* => the name of the autogenerated project
  - *git* => initializes a new git empty repository
  - *compiler* => indicates which of the compilers available within `Zork++`
    should be used to set up the template and run the generated binary for the template

- -v => Outputs more information to stdout. The classical `verbose` command line flag

# 📑 C++23 `import std;` feature <a name="import_std"></a>

The `C++23` standard is supposed to come with a nice feature to finally support modules in a real way, that is through `import std;` statement.
This means that the whole standard library will be available as one nice and neat component, and
just declaring the usage in one statement.

But this is not available in every compiler using `C++20`, and at the time of writting, their have only
partial or directly didn't support yet this for `C++23`, but some of them has their kind of workarounds.

In `Zork++`, you have this feature enabled for any OS supported and even using `C++20` if:

- you're working with `Clang` because the `modulemap` feature of `Clang`. So, in your project, you're able to:
  - `import std;`  // preferred way, in line with the C++23 feature
  - `import <system_header_name>;` // individually import into your module some specific system header as a module. Needs a precompilation previous process.

- you're working with `MSVC`, you are able to use `import std.core`, as a compiler specific
feature. But this will allow you to not use `#include` directives and instead, use import statements. 

# 🎈 Developers Guide <a name="devs_guide"></a>

Contribute to `Zork++` is technically, an easy task. You just to open an issue, and document some bug that you discovered, or some feature that you w'd like to have.

If you want to solve the bug or contribute with a new feature by yourself, after create the issue, just fork the repository, link a new branch to your fork from the base branch, and when you're happy with the proposal, open us a PR in order to verify the changes and add them to upstream!

## ☑️ Running the tests

We distinguish two kind of tests in Zork:

### Unit tests

- This one tests directly different parts of the internal work of `Zork++`, trying to make a reliable system to work with.

### Integration tests

- Integration tests in Zork are focused on build different kind of `C++` projects, and those under different operating systems.

All tests are running in different workflows that you can [check out here](
    <https://github.com/zerodaycode/Zork/actions>
).

Alternatively, you can always download the source code and run them in a local environment. You will need to have installed `Rust`, and moving into the `zork++` directory, you just need to run `$ cargo test --all`.

# 📑 TODO ZONE <a name = "todo_zone"></a>

## The things that we desire to implement or upgrade in Zork++

- Support the Intel's C++ compiler (not a priority, but it would be nice)
- Dump the commands generated in a text file different from the caché by a command line order
- Have full support for module (interface or implementation) partitions
- Take an eye on how the compilers are implementing the `C++23` `import std;` feature,
and then include it in `Zork++` by default
- Enable an option in the config file where the user can activate the parsing of the project
for every iteration by reading the data save in the cache and checking the last time that a
file included in the config file has been modified, so we will only be generating commands
for the modified files
- Include and offer tests suites in the project directly. We mean, integrate
third party suites directly in `Zork++`

# ⛏️ Built Using <a name = "built_using"></a>

- [Rust](https://www.rust-lang.org/) - The full code is written on Rust
- [Toml](https://www.python.org/) - We are using `toml` for the program configuration files

## TODO - Motivations for the style, etc

# ✍️ Authors <a name = "authors"></a>

- [@pyzyryab](https://github.com/pyzyryab) - Idea and core work
- [@gbm25](https://github.com/gbm25) - Parsers and testing attribute integration
- [@TheHiddenOnSystem](https://github.com/TheHiddenOnSystem) - Actions for autopublish the releases of the project
- [@foiP](https://github.com/foiP) - Nice code changes and upgrades, specially in the Rust rewrite

See also the list of [contributors](https://github.com/zerodaycode/Zork/contributors) who participated in this project.

# 🎉 Acknowledgements <a name = "acknowledgement"></a>

- This project it's largely inspired in [CMake](https://cmake.org/)
