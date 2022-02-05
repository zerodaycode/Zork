import subprocess
import os

from utils.constants import CONFIGURATION_FILE_NAME
from workspace_scanner import find_config_file
from config_file_parser import get_project_config
from utils.exceptions import NoConfigurationFileFound

# TODO Complete with descriptive log information like OS, timestamp...
# TODO Check for toolchains and compiler installations

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

    ... and so on and so forth

    ///! ---- Available sections and it's properties ----- ///!
    
    Note: There is mandatory and optional sections and properties.

    [#project] <optional_section>
    auto_generate: true
    project_name: <project's_name>

    [#compiler] <mandatory_section>
    cpp_compiler: clang, g++, msbuild <mandatory_property>

    [#language] <mandatory_section>
    cpp_standard: 11, 14, 17, 20, 2x, 2a <mandatory_property>
    cpp_modules_support: true, false

    [#build] <optional_section>
    output_dir: default

"""


if __name__ == '__main__':

    if find_config_file(os.getcwd()):
        # TODO Color logs
        # TODO Enable clang color output
        print(
            "\n[INFO]: Starting a new C++ compilation job with Zork"
        )
        # Gets the configuration parameters for building the project
        config = get_project_config(os.getcwd())

        print(f'\nCompiler: {config.get("compiler")}')
        print(f'Language: {config.get("language")}')
        print(f'Build: {config.get("build")}\n')

        # Generate compiler and linker calls
        subprocess.Popen(
            [
                config.get("compiler").cpp_compiler, 
            ]
        )
    else:
        raise NoConfigurationFileFound
