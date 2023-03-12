# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0] - // TODO

### Feature

### Updates

- Source type has been modified to support individual files, and
sourceset now is a collection of those individuals, non one path or multiple paths
- Non module source files are compiled and assembled without linking now. This
allows us to generate the compile_commands.json for every translation unit
in the program, without grouping them into one big command line that was compiling
assembling and linking.
- Due to the described above, now the main task of the main command line
is to link the generated object files together, introducing the necessary dependencies
- Non module source files now have their explicit types and operations
- Internal deps: toml raised to the 0.7.2, criterion raised to its 0.4.0v

### Fix

- Solved a bug for which the source files was always detected as a glob pattern,
even if they were declared in a non glob form

## [0.7.0] - 2022 - 03 - 01

### Feature

- A module cache has been implemented, in order to bump up the time needed between iterations when
the translation units aren't modified.
Currently works as expected in all the compilers, but not in `Clang` under `Windows`, due to
the manual usage of the module map for featuring the `import std;`

- New command line argument `--clear-cache`, for manually deleting the cache when the user wants.

### Fix

- Solved a bug that was causing C++ modules containing a dot in their module identifier declaration to not be correctly processed by Zork++, causing a compilation error due to incorrect parameters

## [0.6.0] - 2022 - 02 - 28

### Feature

- Allowing the usage of `import <system_module>` with `Clang`, by precompiling
the declared system modules required, just as we were doing with `GCC`

## [0.5.0] - 2022 - 02 - 08

### Feature

- The project is full rewritten in Rust
- We included full support for working with C++ module for the three major compilers
- We started to work in a cache, to track data and possibly, speed up big projects

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

- Correction on the log showed for the executable autorunner and for the tests runner

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
