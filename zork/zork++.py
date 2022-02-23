import subprocess
import os
import time

from config_file_parser import get_project_config

from compiler_tasks import build_project

from utils.workspace_scanner import find_config_file
from utils.exceptions import NoConfigurationFileFound
from utils.logs import initial_log, log_process_result, \
    show_final_config_values


""" A cppy project works reading it's own configuration file.
    The configuration file it's formed by two main type of tokens:

    Section attributes -> [#section_attr]
    Section property   -> <lang>_<option_name>: <value>

    A conjunction between an attribute and it's properties
    it's called a section

    Here is an example:

    ///! zork.conf
    [[#compiler]]
    cpp_compiler: clang

    [[#language]]
    cpp_standard: 20

    [[#executable]]
    executable_name: test1
    sources: *.cpp, src/*.cpp

    ... and so on and so forth

    ///! ---- Available sections and it's properties ----- ///!
    
    Note: There is mandatory and optional sections and properties.

    [[#project]] <optional_section>  # Still non available
    auto_generate: true
    project_name: <project's_name>

    [[#compiler]] <mandatory_section>
    cpp_compiler: clang, g++, msbuild <mandatory_property>

    [[#language]] <mandatory_section>
    cpp_standard: 11, 14, 17, 20, 2x, 2a <mandatory_property>
    cpp_modules_support: true, false  # Still not available

    [[#build]] <optional_section>
    output_dir: default

    [[#executable]] <optional_section>
    executable_name: any str
    sources: the source files target of the compilation process
        Admits wildcard files, even mixed with path and file
    execute_after_build: true

"""


if __name__ == '__main__':
    process_init_time = time.time_ns() // 1_000_000
    initial_log()

    if find_config_file(os.getcwd()):
        # Gets the configuration parameters for building the project
        config = get_project_config(os.getcwd())

        show_final_config_values(config)

        print(
            f'Calling <{config.get("compiler").cpp_compiler}> ' +
            ' to perform the build job'
        )
        process_result = build_project(config)

        log_process_result(process_init_time, process_result)

        # Runs the generated executable if configurated
        if config.get('executable').auto_execute == 'true':
            subprocess.Popen([
                f'./{config.get("build").output_dir}' +
                f'/{config.get("executable").executable_name}'
            ])
    else:
        raise NoConfigurationFileFound
