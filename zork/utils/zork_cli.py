""" _summary_

    Provides the command line interface for be consumed as a client
"""

import argparse
import subprocess

from zork.utils import constants


def command_line_interface():
    """ Manages to take the available Zork program options as
        command line arguments """

    parser = argparse.ArgumentParser(description='The Zork CLI')
    parser.add_argument(
        '-v',
        '--verbose',
        dest='verbose',
        action='store_true',
        help='Controls the information sent to stdout/stderr'
    )

    subparser = parser.add_subparsers(help='New')
    new_arg_parser = subparser.add_parser(
        'new', help="Generates a new C++ project",
    )
    new_arg_parser.add_argument(
        'project_name',
        type=str,
        nargs=1,
        action='store'
    )
    new_arg_parser.add_argument(
        '-l',
        '--legacy',
        dest='legacy',
        action='store_true',
        help='To generate a C++ modules project or a \
            classical one with headers'
    )
    new_arg_parser.add_argument(
        '-g',
        '--git',
        dest='git',
        action='store_true',
        help='Initializes a new local git repo'
    )
    new_arg_parser.set_defaults(new_arg_parser=True)

    return parser.parse_args()


def new_project_autogenerator(
    project_name: str,
    git_repo: bool,
    cpp_compiler: str
):
    """ Generates a new C++ standarized empty base project
        with a pre-designed structure to organize the
        user code in a modern fashion way.

        Base design for create the project files and folders:
            - ./include/<project_name>
                - hello_zork.<mod_extension>
            - ./src/<project_name>
                - hello_zork.cpp
            - test
            - dependencies
    """

    if git_repo:
        subprocess.Popen([
            'git', 'init', '.'
        ])

    # Generates the zork.conf file
    with open('zork.conf', 'w') as zork_conf_file:
        zork_conf_file.write(constants.ZORK_CONF_AUTOG)

    subprocess.Popen([
        'mkdir', project_name
    ])
    # Generates the main.cpp file
    with open('main.cpp', 'w') as main_cpp_file:
        main_cpp_file.write(constants.MAIN_CPP)

    subprocess.Popen([
        'mkdir', f'{project_name}/include/{project_name}'
    ])
    # subprocess.Popen([
    #     'touch',
    #     f'{project_name}/include/{project_name}/sum.' +
    #     f'{"cppm" if cpp_compiler == "clang" else "ixx"}'
    # ])

    subprocess.Popen([
        'mkdir', f'{project_name}/src'
    ])
    subprocess.Popen([
        'mkdir', f'{project_name}/src/{project_name}'
    ])
    # Generates a module source file
    file_path: str = f'{project_name}/src/{project_name}/math_file'
    file_ext: str = "cppm" if cpp_compiler == "clang" else "ixx"
    with open(f'{file_path}.{file_ext}', 'w') as src_mod_file:
        src_mod_file.write(constants.SRC_MOD_FILE)

    subprocess.Popen([
        'mkdir', f'{project_name}/testing'
    ])
    subprocess.Popen([
        'mkdir', f'{project_name}/dependencies'
    ])
