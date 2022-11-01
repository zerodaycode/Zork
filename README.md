<p align="center">
  <a href="" rel="noopener">
 <img width=300px height=200px style="border-radius: 50%" src="./assets/zork++_logo.png" alt="Zork++ Logo"></a>
</p>

<h1 align="center">The Zork++ project</h1>

<h3 align="center"> A modern C++ project manager and build system for modern C++
</h3>
</br>

<div align="center">

[![Pylint](https://github.com/zerodaycode/Zork/actions/workflows/pylint.yml/badge.svg?branch=main)](https://github.com/zerodaycode/Zork/actions/workflows/pylint.yml)
[![GitHub Issues](https://img.shields.io/github/issues/zerodaycode/Zork.svg)](https://github.com/zerodaycode/Zork/issues) </br>
[![GitHub Pull Requests](https://img.shields.io/github/issues-pr/zerodaycode/Zork.svg)](https://github.com/zerodaycode/Zork/pulls)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](/LICENSE)
[![Windows Installer](https://github.com/zerodaycode/Zork/actions/workflows/windows-create-installer.yml/badge.svg?branch=main)](https://github.com/zerodaycode/Zork/actions/workflows/windows-create-installer.yml)
![Coverage](./assets/coverage-status.svg)
</div>

---
</br>

# 📝 Table of Contents

- [About](#about)
- [Getting Started](#getting_started)
- [The `zork.conf` quick start](#usage)
- [The `zork.conf` reference guide](#zork_conf_reference)
- [Windows special requirements](#windows_special_requeriments)
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

Easiest way to start with `Zork++` is to download the [latest release
available](https://github.com/zerodaycode/Zork/releases) for your current operating system. After that, `Zork++` will be ready to use in your system.

## - Availability

`Zork++` is available for:

- Linux (tested on Arch and Debian systems)
- Windows (there's some limitations)
- MacOS (still not complete support)

## -  Prerequisites

In order to work with `Zork++`, you will need a `C++` toolchain and a compiler in your system.

- Clang
- GNU GCC
- Microsoft MSVC

Currently, `Zork++` it's only working with `LLVM's Clang`, but is ready to implement the other two major compilers. Please see the [contributing guide](CONTRIBUTING.md) if you're ready to help us!

> NOTE: If you are using Windows, you will need to install Clang via `Msys2`. See [Windows special requeriments](#windows_special_requeriments) entry for more info

## - Generating a new C++ project

You can use Zork with an already existing project, or start a new one from a terminal.

- Choose an empty folder to kick off with a fresh start.
- Assuming, you have `Zork++` in your path (installers should have it done for you), type:

`$ zork++ new calculator --compiler clang`

After a moment, a new project should be created in your desired directory.
And a output should appear in your terminal.

```
[SUCCESS]: Compilation job finished. Total time spent on the process: 332 ms

Hello from an autogenerated Zork project!
RESULT '+': 10
RESULT '-': 6
RESULT '*': 16
RESULT '/': 1
```

What happened here?

- We are calling the `Zork` executable previouslly installed on the system
- With the new argument, we are instructing `Zork` to autogenerate a new `C++` project where the standard is `C++20`, an it's built based on *modules*. That project contains a predeterminated structure as shown below

<p align="center">
  <a href="" rel="noopener">
 <img width=250px height=300px src="./assets/autogenerated_project_structure.png" alt="Zork++ Logo"></a>
</p>

We created a folder `examples`. Then, we ran the command line above to create the new project `calculator`. Zork autogenerated a structure where:

- `dependencies` => Empty folder. We recommend you to put your third-party or external libraries, headers, etc. , in this folder.
- `ifc` => stands for *interfaces*. We put here the *module interface units* of the project.
- `src` => where the *module implementation files* live.
- `tests` => where the *unit tests* and the *integration tests* of your new project are.
- `main.cpp` => The classical entry point for a `cpp` program.
- `zork.conf` => Finally! We arrived at the most important part that you will need to understand when working with `Zork++`.
See the [The zork.conf config file](#zork_conf) section to have a better understanding on how to write the configuration file and your project.

> Note that this structure is just a guideline. You may prefer to organize your files in a complete different way. We are just providing a predefined standard to quickly start working with it.

Let's explore a little bit the `out` directory. Here is where your compiler will place all the elements after the compilation and linkage process. It may contain a binary, a library... Also, contains the intermediate files needed to create the final executable or library.

- `modules` => it's a generated folder by `Zork` where the compiler will place all the precompiled module interfaces. Also, is where it stores the object files for the `cpp` source files.
- `zork/intrinsics` => this is a special one. Sometimes `Zork` needs additional things to work properly. So this is the place where those neccesary things live. See [Windows special requeriments](#windows_special_requeriments) for more info.
- `calculator.exe` => We have arrived! This is the final binary generated for the project.

# 🔧 The Zork's `zork.conf` config file <a name="usage"></a>

Wow! We finally arrived at the most important part of `Zork++`. The `zork.conf` configuration file is the core of the project. This is where you define the set-up of your project, instructing `Zork++` to behave as you expect. We are going to take the `zork.conf` from the example of the last entry, but first, let's clarify the semantics of the file.

- `[[#attribute]]` => This is an attribute. An attribute within `Zork++` is like a major region where you will configure certain aspects related with the attribute's name.
- `property: value` => A property always belongs to an attribute. This is where you specify its value.

> The conjunction of an `[[#attribute]]` with its declared `properties` is known as a section.

Here is the configuration file for the example above:

```
# This file it's autogenerated as an example of a Zork config file

[[#project]]
name: calculator
authors: Zero Day Code  # Replace this for the real authors

[[#compiler]]
cpp_compiler: clang++

[[#language]]
cpp_standard: 20
std_lib: libc++
modules: true

[[#build]]
output_dir: ./out

[[#executable]]
executable_name: calculator
sources: *.cpp
auto_execute: true

[[#modules]]
base_ifcs_dir: calculator/ifc/
interfaces: *.cppm
base_impls_dir: calculator/src/
implementations: 
    math.cpp
    math2.cpp=[math]
```

As you see, there is a total of 6 sections. Every section starts always with an `[[#attribute]]` and is followed by its properties.

> Look at the `*.file_ext` syntax. This is called *wildcard syntax*. Wildcard syntax will allow you to specify every file in the relative path of that wildcard that has the same extension, avoiding you to write every `.file_ext` file present in that path.

```
*.cpp             // Takes every .cpp file in the root of the project.
./sources/*.cpp   // Takes every .cpp file in the ./sources folder

// [Alternate syntaxes]

// You can specify any infinite number of directories comma separated
sources: *.cpp, ./sources/*.cpp

// or by a new line and a tab indentation
sources: 
    *.cpp
    ./sources/*.cpp
```

> Note that, if either an `[[#attribute]]` or a `property` isn't one of the available inside `Zork++`, the program will refuse to run raising an error.

> For a full reference of every property available under a specific section, please see the [zork.conf](#zork_conf_reference) reference guide.

Let's briefly discuss every section, to get a general perspective of what we are building.

- `[[#project]]` => Here you specify the metadata for the project. The `name` property is an important one, because some parsing sometimes makes it based on this parameter. Caution.

- `[[#language]]` => The configuration for the `C++` compiler options. From the desired language level, to the `std` library vendor dependant that you want to use, to the explicit declaration of use modules (not required, some properties are optional, see the reference for more info.)

- `[[#build]]` => The compiler output files, also with some internal ones needed by Zork. Here you specify where you want to create that directory.

- `[[#executable]]` => Whenever `Zork++` sees this attribute, it knows that he must produce an executable. You must specify the name of the executable, the sources that adjust it and you may optionally specify if you want to automatically run the project after the compilation finishes.

- `[[#modules]]` => The core section to instruct the compiler to work with `C++20 modules`. The most important are the base path to the *interfaces* and *implementation* files (usually you don't want to spread your files elsewhere), so `Zork++` knows where it should look for them, and the `interfaces` and `implementation` files where you speficy exactly what modules are composing your project.

- `[[#tests]]` => Tests attribute allows you to run the tests written for
your application in a convenient way. You only have to provide the sources
and... ready to go! `Zork++` will take care of all the rest.

For now, tests run independent from the executables or library generations. So, if you want to check the health of your applications and run your tests, just invoke Zork's binary and pass to the command line the `--tests` flags.

```
zork++ --tests
```

You manually must provide a tests suite for now. There are some plans to include one or two of the major ones shipped with `Zork++` directly, like `boost::ut` or `Catch2`, but for now, you must manually download the source files and pass them (if applies) to `Zork`.

`base_path` property (optional) allows you to reduce the path needed to write
every time for every source file.

## Additional notes on the `[[#modules]]` properties syntax

> Whenever you declare a module interface or a module implementation in the configuration file, you must take in consideration that sometimes modules (both interfaces or implementations) depend on other modules. Dependencies of one or more modules are declared as shown below:

```
implementations: 
    math.cpp  // Implicitly depends on `math` interface

    math2.cpp=[math]  // Explicitly declares that depends on the `math` interface
```

> If the relation has no declared dependencies,
(meaning that only the file name of the module implementation
is declared here),
`Zork++` will assume that the unique interface that the module implementation depends has the same
name of the module interface unit.

The last one is not true for the *module interface units*. They can take any number of module dependencies as they wish, but there's no direct mapping on the no dependency declaration, because it's an interface unit.

Caution: the *module implementation unit* always depends, at least, on an interface unit, that's why `Zork++` assumes that, if the implementation unit does not explicitly declares an interface unit, an interface unit exists with the same name (without file extension) in the interfaces directory.

If the user does not use the same file name for both
the interface and the declaration, and not direct dependency is declared, a compiler error will be thrown because `Zork++` does not care about wrong
specified dependencies.

An alternative syntax exists to declare module and module dependencies (both for interfaces and implementation units)

```
implementations: math.cpp; math2.cpp=[math]
```

This allows you to declare every file that you desire in a unique line (someone could prefer it), but it must be separated with `;` in order to work.

> This one is the initial `Zork++` syntax for declaring source files dependencies, is considered the legacy way, and we reserve ourselves the
right of removing them from the application without previous warning or notification.

# 📑 The `zork.conf` reference guide <a name="zork_conf_reference"></a>

### Here we will list all the sections, with the attributes and its properties available on Zork, indicating when an attribute or a property is optional or mandatory, and for the properties, if they have default values

```
PROJECT_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#project]]',
    mandatory=True,
    properties=[
        Property(
            identifier='name',
            mandatory=True,
            values=Any
        ),
        Property(
            identifier='authors',
            mandatory=False,
            values=Any
        )
    ]
)

COMPILER_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#compiler]]',
    mandatory=True,
    properties=[
        Property(
            identifier='cpp_compiler',
            mandatory=True,
            values=SUPPORTED_COMPILERS
        ),
        Property(
            identifier='system_headers_path',
            mandatory=False,
            values=Any
        )
    ]
)

LANGUAGE_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#language]]',
    mandatory=True,
    properties=[
        Property(
            identifier='cpp_standard',
            mandatory=True,
            values=SUPPORTED_CPP_LANG_LEVELS
        ),
        Property(
            identifier='std_lib',
            mandatory=False,
            values=SUPPORTED_CPP_STDLIBS
        ),
        Property(
            identifier='modules',
            mandatory=False,
            values=['True', 'true']
        ),
    ]
)

BUILD_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#build]]',
    mandatory=False,
    properties=[
        Property(
            identifier='output_dir',
            mandatory=False,
            values=Any
        )
    ]
)

EXECUTABLE_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#executable]]',
    mandatory=False,
    properties=[
        Property(
            identifier='executable_name',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='sources_base_path',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='sources',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='auto_execute',
            mandatory=False,
            values=['true', 'false']
        )
    ]
)

MODULES_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#modules]]',
    mandatory=False,
    properties=[
        Property(
            identifier='base_ifcs_dir',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='interfaces',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='base_impls_dir',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='implementations',
            mandatory=False,
            values=Any
        )
    ]
)

TESTS_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#tests]]',
    mandatory=False,
    properties=[
        Property(
            identifier='tests_executable_name',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='sources_base_path',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='sources',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='auto_run_tests',
            mandatory=False,
            values=['true', 'false']
        ),
    ]
)
```

where, the constant values declared for some *values* properties are:

```
SUPPORTED_CPP_LANG_LEVELS: list = [
    '11', '14', '17', '20', '23', '1a', '2a', '1x', '2x'
]

SUPPORTED_CPP_STDLIBS: list[str] = ['stdlibc++', 'libc++']

SYSTEM_HEADERS_EXPECTED_PATHS: list[str] = ['C:/msys64/mingw64/include/c++']
```

> Note that in the *values* properties that are declared as a Python's array, we're not expecting you to declare an array. We are expecting
one of the `allowed properties` declared within that array.

# 💻 Windows special requeriments <a name="windows_special_requeriments"></a>

> If you are using Windows, you will need to install Clang via [Msys2](https://www.msys2.org/) in order to make it work with GCC's STD. Unfortunately we are not supporting the Microsoft tools directly, the only way is to link against GCC `(target=x86_64-windows-gnu-pc)`

> Due to the above explained issue, `Zork++` is specting that you have installed your `msys2` system in the root of your `C:` hard drive.
Potencially, we will upgrade this restriction to a better dynamic way in order for it to find the required things on Windows, but for now, it isn't a priority for the dev team, because we're only supporting `Clang`, and `Clang` isn't self-sufficient on Windows (depends either on `GCC` or `MSVC` tools).

Also, there are troubles mapping standard library headers to module units, due to redefinition conflicts, so not everything works always in a smooth way under Windows.

However, if you are using the `import std` feature, you should not have any problem building up your project.

# 📑 C++23 `import std;` feature <a name="import_std"></a>

The `C++23` standard is supposed to come with a nice feature to finally support modules in a real way, that is through `import std;` statement.

This means that you will never have to `#include <header_name>` in your project to have access to some standard library utility. Also, there's a syntax in some compilers like `import <iostream>;` (any compiler offers its own alternate way of doing this) in order to be able to use the standard library headers as a module unit.

In `Zork++`, you have this feature enabled for any OS supported and even using `C++20` if you're working with `Clang` (remember, it's the unique compiler support that we are offering now at the writting time), because the `modulemap` feature of `Clang`, that translates system headers into modules. So, in your project, you're able to:

- `import std;`  // preferred way, in line with C++23 feature
- `import <system_header_name>;` // individually import into your module some specific system header as a module.

> Note: the last syntax only works under Linux environments (potentially MacOS too). Under Windows, you will only be able to use `import std;` because `Zork++` manually implements a specific module map file to bring you support for the most important system headers. This is a temporary workaround (that's what we are expecting) and in future releases this will work evenly for any OS.

# 🎈 Developers Guide <a name="devs_guide"></a>

## TODO doc for future contributors

Add notes about how to use the system. This is a todo-ed entry.
Keep in mind that we may take a few days to deploy this guide.

## ☑️ Running the tests

We distinguish two kind of tests in Zork:

### Unit tests

- This one tests directly different parts of the internal work of Zork, trying to make a reliable system to work with.

### Integration tests

- Integration tests in Zork are focused on build different kind of `C++` projects, and those under different operating systems.

All tests are running in different workflows that you can [check out here](
    https://github.com/zerodaycode/Zork/actions
). But, also, you can always download the source code and run them in a local environment.

# TODO ZONE <a name = "todo_zone"></a>

### Things that we desire to implement or upgrade in Zork++

- Recognize multiple *zork.conf* files by suffix. For ex: `zork_windows.conf` and
`zork_linux.conf`. This will allow the user to have multiple conf file for different platforms when options are susceptible to change (use libc++ under Linux and libstdc++ under Windows witn Clang). Also, we can start to think in the same pattern for environments (PRE, PRO...) and allow things like (zork_linux_pro.conf)

# ⛏️ Built Using <a name = "built_using"></a>

- [Python](https://www.python.org/) - Language

## TODO - Motivations for the style, etc

# ✍️ Authors <a name = "authors"></a>

- [@pyzyryab](https://github.com/pyzyryab) - Idea and core work
- [@gbm25](https://github.com/gbm25) - Parsers and testing attribute integration
- [@TheHiddenOnSystem](https://github.com/TheHiddenOnSystem) - Actions for autopublish the releases of the project

See also the list of [contributors](https://github.com/zerodaycode/Zork/contributors) who participated in this project.

# 🎉 Acknowledgements <a name = "acknowledgement"></a>

- This project it's largely inspired in [CMake](https://cmake.org/)
