#!/bin/sh

# This file provides command alias to the developers involved in Zork to avoid the usage of a bunch
# of command.
# In order to run the script, simply type `$ . ./alias.sh` from the root of the project.
# (refreshing the current terminal session could be required)

# Builds the project and sets-up the examples folder, which also serves as a kind
# of integration test to check that the generated C++ modules examples works well in the current
# host.
alias CompileZork='rm -rf examples && mkdir examples/ && pyinstaller ./zork/zork++.py --onefile && cp ./dist/zork++ ./examples && cd examples && ./zork++ -v new calculator --compiler clang && cd .. && rm -rf build && rm -rf dist && rm zork++.spec'

# Runs the tests with pytest, showing the results and the code coverage on the terminal
alias RunTests='python -m pytest --emoji -vv --cov zork --cov-branch --cov-report term-missing'

# Quick way of compile Zork++, and see a Zork++ build process for an example project (if it's already created)
alias TestZorkBuild='cargo build && cp ./target/debug/zork.exe . && ./zork.exe -vv build'
# Same as above, but with the 'run' command
alias TestZorkRun='cargo build && cp ./target/debug/zork.exe . && ./zork.exe -vv run'
