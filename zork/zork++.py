""" A Zork project works reading it's own configuration file.
    The configuration file it's formed by two main type of tokens:

    Section attributes -> [#section_attr]
    Section property   -> <lang>_<option_name>: <value>

    A conjunction between an attribute and it's properties
    it's called a section

    Here is an example:

    ///! zork.conf
    [[#project]]
    name: example

    [[#compiler]]
    cpp_compiler: clang

    [[#language]]
    cpp_standard: 20

    [[#executable]]
    executable_name: test1
    sources: *.cpp, src/*.cpp

    ... and so on and so forth
"""

import os
import time

from config_file_parser import get_project_config

from compiler_tasks import build_project

from utils.workspace_scanner import find_config_file
from utils.exceptions import NoConfigurationFileFound
from utils.logs import initial_log, log_process_result, \
    show_final_config_values
from utils.zork_cli import command_line_interface, new_project_autogenerator
import runners


if __name__ == '__main__':
    process_init_time = time.time_ns() // 1_000_000

    cli_options = command_line_interface()
    verbose: bool = cli_options.verbose
    tests: bool = cli_options.tests

    initial_log(tests)

    if 'project_name' in cli_options:
        proj_already_created: bool = \
            os.path.exists(cli_options.project_name[0])

        if cli_options.compiler and not proj_already_created:
            new_project_autogenerator(
                cli_options.project_name[0],
                cli_options.git,
                cli_options.compiler[0]
            )
        else:
            new_project_autogenerator(
                cli_options.project_name[0],
                cli_options.git,
                'clang'
            )

    if find_config_file(os.getcwd(), verbose):
        # Gets the configuration parameters for building the project
        config = get_project_config(os.getcwd(), verbose)

        # Project name definition
        proj_name: str = cli_options.project_name[0] \
            if 'project_name' in cli_options \
            else config.get('project').name

        if verbose:
            show_final_config_values(config,)
            print(
                f'Calling <{config.get("compiler").cpp_compiler}> ' +
                ' to perform the build job'
            )

        # inlined call to build the project and log the result
        log_process_result(
            process_init_time,
            build_project(
                config,
                verbose,
                tests
            )
        )

        # Runs the generated executable if configurated
        if config.get('executable').auto_execute == 'true':
            runners.run_executable(config, proj_name)
    else:
        raise NoConfigurationFileFound
