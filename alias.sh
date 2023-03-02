#!/bin/sh

# Quick way of compile Zork++, and see a Zork++ build process for an example project (if it's already created)
alias TestZorkBuild='cargo build && cp ./target/debug/zork.exe . && ./zork.exe -vv build'
# Same as above, but with the 'run' command
alias TestZorkRun='cargo build && cp ./target/debug/zork.exe . && ./zork.exe -vv run'
