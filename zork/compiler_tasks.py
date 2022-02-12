"""[summary]

    This file provides several functions that creates the 
    command line compiler calls, generated after parsing the 
    Zork config file and retrieve the data
"""

import subprocess

from program_definitions import CLANG, GCC, MSVC
from utils.exceptions import UnsupportedCompiler

def build_project(config: dict):
    """ Calls the selected compiler to perform the build of the project """
    compiler = config['compiler'].cpp_compiler

    if compiler == CLANG:
        call_clang_to_compile(config)
    elif compiler == GCC:
        raise UnsupportedCompiler(GCC)
    else: 
        raise UnsupportedCompiler(MSVC)

def call_clang_to_compile(config: dict):
    """ Calls Clang++ to compile the provide files / project """
    # Generate compiler and linker calls
    command_line = [
        config.get("compiler").cpp_compiler, 
        '--std=c++' + config.get("language").cpp_standard,
        '-stdlib=' + config.get("language").std_lib,
        '-o', config.get("executable").executable_name,
    ]

    for source in config.get("executable").sources:
        command_line.append(source)

    print(f'Command line executed: {" ".join(command_line)}')
    # command_line_as_str = " ".join(command_line)
    # print(f'AS STR: {command_line_as_str}')
    subprocess.Popen(
        command_line
    )

    # TODO Add the include path option to the (?compiler attribute)
    # TODO Analize the output of subprocess
    # TODO NEW FEATURE Add wintoast10 (linux?) to notify the end of the build