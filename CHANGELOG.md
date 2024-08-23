# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.11.0] - 2024 - 08 - 23

### Features

- Added support for `Clang >=18` to use the standard library modules (std and std.compat).
- Added a new cfg property under the [[compiler]] attribute to manually set where the `libc++` installation lives
- If the property above isn't present, `Zork++` will try to find a suitable candidate automatically. This can be hazardous
if the user has different installations of `libc++`, but that's why exists the property describe in the previous point.
- 'import std' feature is yet available for `clang < 18` via clang modules (instead of std modules) and modulemaps. Anyway,
we recommend to use Clang version that uses `std modules` for better support. This feature is kind of broken on Windows, and
we're not sure if we inteend to fix it (we may delete this feature in future releases)

### Misc and or internal

- Added live tests on GitHub's virtual machines to test that `Zork++` effectively builds more complex projects, like
the [Zero library](https://github.com/zerodaycode/Zero). This is poiting towards a development branch, and when everything
is stabilized will point towards the default branch (main)

- The `flyweight` data factory (the one that creates the types that holds the more repeated arguments across the user build) has
been redesigned to be more efficient and readable.

## [0.10.4] - 2024 - 08 - 17

### Fixes

- Allowing the user to declare absolute file paths in the configuration file 

## [0.10.3] - 2024 - 08 - 12

### Fixes

- The "compile commands" of the generated compilation database was being generated without including the driver

## [0.10.2] - 2024 - 08 - 10

### Fixes

- The `driver-path` property from the configuration file wasn't being picked when declared

## [0.10.1] - 2024 - 08 - 10

### Internal 

- GitHub code coverage actions

## [0.10.0] - 2024 - 08 - 07

### Features

- **Breaking** - *Targets*: `Executable` and `Tests` toml entries are removed in favour of `[targets.<target_identifier>]` entries.
Each targets allow the user to build *N* independent final products, being these `binaries (executables)` for now, while static
and dynamic libs will be implemented in the upcoming releases

- NOTE: A **target identifier** is the string value after the `.` on any `[targets.<target_identifier>]`

- Added a `--targets` CLI flag to manually specify what targets wants the user to be processed in the current invocation.
Ex: `--targets target1,target2,tests1,tests2,tests3`

### Changes

- `Tests`: `test` command now only runs those targets which they contain the string `test` in its **target identifier**

- **breaking** - `project_root` property (under the `[compiler]` attribute) is renamed to `code_root`

### Performance

- The codebase suffered a major reorganization and re-factorization. These are all internal changes that aren't
exposed through the public API. The most notorious points are:
  - The introduction of *flyweights* data-structures, that
  allows to reduce the program's memory footprint dramatically, and also the size of the generated cache files, being only
  created once and then only joined for being passed into iterable views that makes the full command line of every translation
  unit.
  - The amount of required code lines that was basic doing the same job generating arguments for every different
  kind of translation unit
  - All the generated commands are now stored in the cache as a separated entity, and they are only regenerated
  if the translation unit was modified since the last program run
  - The project model is now cached, and only rebuilt if the configuration file changes between different program iterations
  - All the translation units are now processed in only one unique procedure, since they are managed as trait objects
  in the main functions of the commands generation, and then small helpers creates, depending on the kind of translation unit
  processed the different arguments required for the source file

### Misc and or Internal
  - Several internal APIs that uses helpers have received new unit tests, to ensure the robustness of their job
  - There's a lot of legacy code removed, only maintained for backwards-compatibility reasons.
  - `System headers (GCC and Clang)` that are importable translation units as modules are now translation units as well
  - We managed to satisfy the *Rust* borrow checker while we use the project model as read-only data, while the cache
   is handled exclusively via `&mut` (mutable references), allowing the codebase to be extremely fast and performant
  - Other minor optimizations has been applied to procedures. We remove a lot of allocation points thanks to the newly introduced
   `clone-on-write` idiom, to handle the data that comes borrowed since the configuration file up until the end of the process

## [0.9.0] - 2024 - 05 - 25

### Feature

- `MSVC` is now fully compatible with `import std`

## [0.8.8] - 2024 - 04 - 13

### Bug

- Fix a missing cmd flag when building *system modules* with `GCC`, thanks to @Property404

## [0.8.7] - 2023 - 10 - 27

### Feature

- Added a new **CLI flag** to specify where Zork++ should start to work
- Removed the `-fmodules-ts` Clang's command line flag, since it's deprecated since **Clang 16,** and it will
be removed in **Clang 17**. Also, is actions are implied by set the **C++ standard version > 20**.

### Bug
- Corrected the version declared for the project, and now is aligned in every place that is declared with the correct
`Zork++` version

## Update
- README description contains now a warning that **libc++** must be installed in Unix like systems to correctly be able
to use `import std`;

## [0.8.6] - 2023 - 04 - 05

### Feature

- New config file entry parameter for the compiler key, to allow the user to specify the invokable name
of the compiler's driver via CMD

## [0.8.5] - 2023 - 04 - 05

### Feature

- Added a command line argument named `--match-files` that filters all the detected configuration
files for the project by checking if the value of the argument is a substring of the filename of
every config file.


## [0.8.4] - 2023 - 04 - 02

### Fix

- Shortened the error in the command line when a build action fails
- Removed the duplicated error shown in the terminal when the build fails

## [0.8.3] - 2023 - 03 - 16

### Feature

- The modules key of the configuration file got a new property for adding extra
arguments to the command lines generated for build the `C++ modules`

### Fix

- Sources and module implementation translation units wasn't receiving the general
extra compiler arguments

## [0.8.2] - 2023 - 03 - 16

### Update

- Files and directories needed by the program aren't regenerated for every iteration
if they are already present in the filesystem
- Full path for the Clang's Windows modulemap

### Internal

- `Arguments` is an strong type for `Vec<Arguments>`, providing it's own
  convenient API

## [0.8.1] - 2023 - 03 - 15

### Feature

- Constructed the full path for the output directory from the one specified or defaulted
in the configuration

### Update

- Removed the restriction that doesn't allow the user to be able to link against `libc++`
in Windows

### Upgrade
- The absolute paths for all the declared files is preloaded from the declared root from
the project, avoiding make a .canonicalize() call for every one, raising the performance
of the project model build process

## [0.8.0] - 2023 - 03 - 12

### Feature

- Zork++ generates now a compilation database for `C++` projects, known as the `compile_commands.json`
file, which is used by static analyzers and IDE's to offer code completion and linting.
- The cache has been updated to store data in a more efficient layout.
- The overall performance of the cache process has been reviewed. We get rid of a cache clone that was affecting
the performance, and making a huge impact the memory needed for the cache process during runtime
by a factor of two.
Now everything is smoothly handled by mutable and immutable reference types.
- A command line flag `-c` has been included to reset the cache when the user requires.

### Update

- The source type has been modified to support individual files, and
sourceset now is a collection of those individuals, non one path or multiple paths
- Non module source files are compiled and assembled without linking now. This
allows us to generate the compile_commands.json for every translation unit
in the program, without grouping them into one big command line that was compiling
assembling and linking.
- Due to the described above, now the main task of the main command line
is to link the generated object files together, introducing the necessary dependencies
- Non module source files now have their explicit types and operations
- Internal dependencies: criterion raised to its 0.4.0v
- We've merged some parts of the source code that was performing similar operations, specially the one that was mapping
  data in the cache to some other datastructures. Even that technically that parts wasn't exactly
  duplicated code, we've managed to make them cleaned and shorter.

### Fix

- Solved a bug for which the source files was always detected as a glob pattern,
even if they were declared in a non glob form

## [0.7.0] - 2023 - 03 - 01

### Feature

- A module cache has been implemented, in order to bump up the time needed between iterations when
the translation units aren't modified.
Currently works as expected in all the compilers, but not in `Clang` under `Windows`, due to
the manual usage of the module map for featuring the `import std;`

- New command line argument `--clear-cache`, for manually deleting the cache when the user wants.

### Fix

- Solved a bug that was causing C++ modules containing a dot in their module identifier declaration to not be correctly processed by Zork++, causing a compilation error due to incorrect parameters

## [0.6.0] - 2023 - 02 - 28

### Feature

- Allowing the usage of `import <system_module>` with `Clang`, by pre-compiling
the declared system modules required, just as we were doing with `GCC`

## [0.5.0] - 2023 - 02 - 08

### Feature

- The project is full rewritten in Rust
- We included full support for working with C++ module for the three major compilers
- We started to work in a cache, to track data and speed up the compilation times when files doesn't change
over the compilation iterations

## [0.4.2] - 2022 - 12 - 28

### Fix

- Solved a bug that was leading to an incorrect command line generation under Unix OS

## [0.4.1] - 2022 - 12 - 28

### Update

- Upgraded consistency on the executable file extension generation for Windows environments

### Fix

- Correction on the log showed for the executable auto runner and for the tests runner

## [0.3.1] - 2022 - 12 - 06

### Fix

- Correction on the log showed for the executable runner and for the tests runner

## [0.3.0] - 2022 - 11 - 22

### Added

- Upgraded the release action to upload assets with the Linux binary

## [0.2.0] - 2022 - 11 - 20

### Added

- `extra_args` property has been included for the `compiler`,
`executable` and `tests`.

## [0.1.0] - 2022 - 10 - 30

### Added

- This CHANGELOG file to hopefully serve as an evolving example of a
  standardized open source project CHANGELOG.
- Code for the first release of the project
- README now contains the official documentation for the project
- GitHub actions to automate certain processes, like static code analysis,
run ut/integration tests and publish releases.
- A initial distribution for Windows systems is published in ZIP format with
an installer to automate the process. This will install the program in the users's machine, and will set up the PATH environment variable.
