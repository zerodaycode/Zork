#!/bin/sh

# This file provides an alias to the developers involved in Zork to avoid the usage of a bunch
# of commands to build the project and set-up the examples folder, which also serves as a kind
# of integration test to check that the C++ modules examples provides works well in the current
# host.

# In order to run the script, simply type `$. ./alias.sh` from the root of the project.
# Then, the CompileZork alias will be ready to use (refreshing the term session could be required)
alias CompileZork='rm -rf build && rm -rf dist && rm -rf examples && mkdir examples/ && pyinstaller ./zork/zork++.py --onefile && cp ./dist/zork++ ./examples && cd examples && ./zork++ -v new zork_proj_example --compiler clang && cd ..'
