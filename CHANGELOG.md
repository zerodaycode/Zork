# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1] - 2022 - 12 - 28

### Update

- Upgraded consistency on the executable file extension generation for Windows environments

### Fix

- Correction on the log showed for the executable autorunner and for the tests runner

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
