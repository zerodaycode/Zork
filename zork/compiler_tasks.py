""" _summary_

    This file provides several functions that creates the
    command line compiler calls, generated after parsing the
    Zork config file and retrieve the data
"""

import glob
import os
import subprocess
import sys

from program_definitions import CLANG, GCC, MSVC, SYSTEM_HEADERS_EXPECTED_PATHS
from utils.exceptions import LanguageLevelNotEnought, UnsupportedCompiler, \
    NoSystemHeadersFound
from utils import constants


def build_project(
    config: dict,
    verbose: bool,
    tests: bool
) -> int:
    """ Calls the selected compiler to perform the build of the project """

    generate_build_output_directory(config)

    compiler = config['compiler'].cpp_compiler
    command_line: list = []

    if compiler == CLANG:
        command_line = call_clang_to_work(config, verbose, tests)
    elif compiler == GCC:
        raise UnsupportedCompiler(GCC)
    else:
        raise UnsupportedCompiler(MSVC)

    if verbose:
        print(f'The executable command line: {" ".join(command_line)}')
    return run_subprocess(subprocess.Popen(command_line).wait())


def call_clang_to_work(
    config: dict,
    verbose: bool,
    tests: bool
) -> str:
    """ Calls Clang++ to compile the provide files / project """
    # Generates the compiler and linker calls
    base_command_line: list = clang_base_command_line(config)
    base_command_line = base_command_line +\
        config.get('compiler').extra_args

    # The command line with the args for the executable (main executable or
    # tests executable)
    extra_args = config.get('executable').extra_args if not tests\
        else config.get('tests').extra_args
    executable_name = config.get("executable").executable_name if not tests\
        else config.get('tests').tests_executable_name

    command_line = base_command_line + extra_args + [
        '-o',
        f'{config.get("build").output_dir}/'
        + executable_name
        + ('.exe' if constants.OS == constants.WINDOWS else '')
    ]

    # Adds the source files to the command line
    add_sources(config, tests, command_line)

    # Generates a compiler call to prebuild the module units, in case that
    # the attribute it's present, have a valid path to the .cppm module units
    # and the language level it's at least, c++20.
    if config['language'].modules is True:
        if int(config.get('language').cpp_standard) < 20:
            raise LanguageLevelNotEnought(
                20,
                config.get('language').cpp_standard,
                "Modules"
            )

        prebuild_modules_path, interfaces = _clang_prebuild_module_interfaces(
            config, verbose, base_command_line
        )
        implementations = _compile_module_implementations(
            config, verbose, prebuild_modules_path, base_command_line
        )

        for module_ifc in interfaces:
            command_line.append(module_ifc)
        for module_src in implementations:
            command_line.append(module_src)

        command_line.append(
            f'-fprebuilt-module-path={prebuild_modules_path}'
        )

    return command_line


def add_sources(config: dict, tests: bool, command_line: list[str]):
    """ Adds to the command line the files found on the
        executable and tests attributes under the sources
        property.
    """
    tar = config.get('executable') if not tests else config.get('tests')
    sources, base_path = (tar.sources, tar.sources_base_path)

    if base_path != '':  # Adding a final slash to the base path
        base_path = f'{base_path}/'

    for source in sources:
        if '*.' in source:
            for wildcard_ifc in glob.glob(source):
                command_line.append(
                    base_path + wildcard_ifc.replace('\\', '/')
                )
        else:
            command_line.append(base_path + source)


def clang_base_command_line(config: dict) -> list[str]:
    """ Builds a base command line with the common shared
    arguments for every possbile action (build executable,
    build tests, etc.)

    Returns:
        list[str]: _description_
    """
    base_command_line: list[str] = [
        config.get('compiler').cpp_compiler,
        f'-std=c++{config.get("language").cpp_standard}',
    ]

    if constants.OS == constants.WINDOWS:
        if config.get('language').modules is True:
            base_command_line.append('-fimplicit-modules')
            base_command_line.append(
                f'-fmodule-map-file={config.get("build").output_dir}' +
                    '/zork/intrinsics/zork.modulemap'
            )
            # base_command_line.append('--target=x86_64-w64-windows-gnu')
            ## TODO --target should be a configuration option (optional, with base defaults by OS)

    else:
        base_command_line.append('-stdlib=' + config.get('language').std_lib)
        if config.get('language').modules is True:
            base_command_line.append('-fimplicit-modules')
            base_command_line.append('-fimplicit-module-maps')

    return base_command_line


def _clang_prebuild_module_interfaces(
    config: dict,
    verbose: bool,
    base_command_line: list
) -> tuple:
    """ The responsable for generate de module units
        for the C++20 modules feature.
        Returns a list with the args that should be passed into them
        main compiler call in order to enable the modules compilation
        and linkage """
    output_dir: str = config['build'].output_dir
    modules_dir_path = output_dir + '/modules'
    module_ifcs_dir_path = modules_dir_path + '/interfaces'

    if verbose:
        print('\nPrecompiling the module interfaces...')
    # Generate the precompiled modules directory if it doesn't exists
    if 'modules' not in os.listdir(output_dir):
        run_subprocess(subprocess.Popen(['mkdir', modules_dir_path]).wait())
        run_subprocess(
            subprocess.Popen(['mkdir', module_ifcs_dir_path]).wait()
        )

    module_ifcs: list = _get_ifcs(config)

    base_command_line.insert(1, '-c')
    for ifcs_data in module_ifcs:
        # Strips the path part if the module name it's inside a path,
        # (like 'src/inner/module_file_name.cppm') and not alone,
        # as a *.cppm file. Also, strips the file extension for
        # replace it the file name ext for the .pcm one
        module_file: str = ifcs_data[0]
        module_name: str = module_file.split('.')[0]
        if '/' in module_name:
            module_dir_parts_no_slashes: list = module_name.split('/')
            module_name = \
                module_dir_parts_no_slashes[
                    len(module_dir_parts_no_slashes) - 1
                ]

        commands: list = [
            '--precompile',
            '-o',
            f'{module_ifcs_dir_path}/{module_name}.pcm',
            f'./{module_file}'
        ]
        if ".cppm" not in module_file:
            commands.append('-Xclang')
            commands.append('-emit-module-interface')
            commands.append('-x c++-module')

        for ifc_dependency in ifcs_data[1]:
            commands.append(
                f'-fmodule-file={module_ifcs_dir_path}/{ifc_dependency}'
            )

        if verbose:
            print(
                'Module interfaces, command line to execute: ' + \
                " ".join(base_command_line + commands)
            )

        run_subprocess(subprocess.Popen(base_command_line + commands).wait())

    if verbose:
        print('...Precompilation of module interface units finished!\n')

    precompiled_mod_ifcs: list = [
        pmiu.replace("\\", '/') for pmiu in glob.glob(f'{module_ifcs_dir_path}/*.pcm')
    ]

    return module_ifcs_dir_path, precompiled_mod_ifcs


def _get_ifcs(config: dict):
    """ Gets the sources files for the interface files"""
    ifcs_from_config: list = config.get('modules').interfaces
    ifcs: list[tuple[str, list[str]]] = []

    base_ifcs_path: list = config.get('modules').base_ifcs_dir

    if base_ifcs_path != '':
        if base_ifcs_path.endswith('/'):
            base_ifcs_path = base_ifcs_path[:-1]

        for interface_relation in ifcs_from_config:
            ifc_parts = interface_relation.split('=[')
            ifc_file = ifc_parts[0]

            # The interface file may have dependencies
            # or not. So, in Zork, you can declare an interface
            # just by it's file name, or declare some other
            # interface(s) which this interface depends on
            # with the equals to list notation
            #
            # Ex: base_mod.cppm=[mod, mod2, mod3]
            if len(ifc_parts) > 1:
                # The dependencies attached to the current
                # module interface unit
                dependencies = ifc_parts[1].replace(']', '').split(',')

                parsed_deps: list = []
                for interface_dep in dependencies:
                    parsed_deps.append(f'{interface_dep.strip(" ")}.pcm')
                ifcs.append((
                    f'{base_ifcs_path}/{ifc_file}',
                    parsed_deps
                ))
            else:
                if'*.' in ifc_file:
                    for wildcard_ifc in glob.glob(f'{base_ifcs_path}/{ifc_file}'):
                        wildcard_ifc = wildcard_ifc.replace("\\", "/")
                        ifcs.append((f'{wildcard_ifc}', []))
                else:
                    ifcs.append((f'{base_ifcs_path}/{ifc_file}', []))
    else:
        pass
        # TODO Custom error or default value

    return ifcs


def _compile_module_implementations(
    config: dict,
    verbose: bool,
    module_ifcs_dir_path: str,
    base_command_line: list
):
    """
        Compiles the module implementation units, when the declaration
        it's splitted from the implementation, usually in interface file
        and implementation file.

        This process needs to know about the prebuild module interface
        units, and point the implementation unit to the correct
        module interface file.
    """
    output_dir: str = config['build'].output_dir
    modules_dir_path = output_dir + '/modules'
    module_impls_dir_path = modules_dir_path + '/implementations'

    # Generate the precompiled modules directory if it doesn't exists
    if 'modules' in os.listdir(output_dir) and \
        'implementations' not in os.listdir(modules_dir_path):
        run_subprocess(
            subprocess.Popen(['mkdir', module_impls_dir_path]).wait()
        )
    if verbose:
        print('Compiling the module implementations...')

    module_impls_relations: list = _get_impls(config)

    for module_impl_tuple in module_impls_relations:
        commands: list = []

        module_impl = module_impl_tuple[0]

        # Generates the path for the special '**' Zork syntax
        commands.append(module_impl.replace('\\', '/'))
        mod = module_impl \
            .replace('\\', '/') \
            .split('/')

        mod2 = mod[(len(mod) - 1)] \
            .split('.')[0]

        commands.append('-o')
        commands.append(f'{module_impls_dir_path}/{mod2}.o')

        for ifc_dependency in module_impl_tuple[1]:
            commands.append(
                f'-fmodule-file={module_ifcs_dir_path}/{ifc_dependency}'
            )
        if verbose:
            print(
                'Module implementation units, command line to execute: ' + \
                " ".join(base_command_line + commands)
            )
        run_subprocess(subprocess.Popen(base_command_line + commands).wait())

    if verbose:
        print('...\nModule implementation units compilation finished!\n')

    return [
        pmiu.replace("\\", '/') for pmiu in glob.glob(f'{module_impls_dir_path}/*.o')
    ]


def _get_impls(config: dict):
    """ Gets the sources files for the module implementation files
        and the interfaces that the implementation files depends on
    """
    impls_from_config: list = config.get('modules').implementations
    impls: list = []

    base_impls_path: list = config.get('modules').base_impls_dir

    if base_impls_path != '':
        if base_impls_path.endswith('/'):
            base_impls_path = base_impls_path[:-1]

        for impl_relation in impls_from_config:
            impls_parts = impl_relation.split('=[')
            impl_file = impls_parts[0].split('.')

            # If the relation has no list with the dependencies,
            # so only the file name of the module implementation
            # is declared here,
            # Zork will assume that the unique interface that the
            # current module implementation depends has the same
            # name of the implementation unit.
            #
            # If the user does not use the same file name for both
            # the interface and the declaration, a compiler error
            # will be throwed.
            if len(impls_parts) > 1:
                # The extension of the source file
                impl_file_ext = impl_file[1]
                # The dependencies attached to the current
                # module implementation unit
                dependencies = impls_parts[1].replace(']', '').split(',')

                parsed_deps: list = []
                for interface in dependencies:
                    parsed_deps.append(f'{interface.strip(" ")}.pcm')
                impls.append((
                    f'{base_impls_path}/{impl_file[0]}' +
                        f'.{impl_file_ext}',
                    parsed_deps
                ))
            else:
                impls.append((
                    f'{base_impls_path}/{impl_file[0]}' +
                        f'.{impl_file[1]}',
                    [f'{impl_file[0]}.pcm']
                ))
    else:
        pass
        # TODO Raise error or generate base default path
        # base def path, eg. './<proj_name>/src/'

    return impls


def generate_build_output_directory(config: dict):
    """ Creates the directory where the compiler will dump the
        generated files after the build process.

        Also, it will generate the [output_build_dir/zork/intrinsics],
        which is the place where Zork dumps the things that needs to
        work under different conditions. For example, currently under
        Windows, modules needs to be mapped to it's custom modulemap
        file in order to use import statements with the system headers.
    """
    output_build_dir: str = config['build'].output_dir
    zork_intrinsics_dir: str = f'{output_build_dir}/zork/intrinsics'

    if not output_build_dir.strip('./') in os.listdir():
        """ Kind of cache feature. If the directory exists,
        we don't regenerate it on every compilation """
        run_subprocess(subprocess.Popen(['mkdir', output_build_dir]).wait())
        if constants.OS == constants.WINDOWS:
            run_subprocess(subprocess.Popen(['mkdir', '-p', zork_intrinsics_dir]).wait())
            generate_import_std(config, zork_intrinsics_dir)
    else:
        if constants.OS == constants.WINDOWS:
            if not zork_intrinsics_dir.strip('./') in os.listdir():
                run_subprocess(subprocess.Popen(['mkdir', '-p', zork_intrinsics_dir]).wait())
                generate_import_std(config, zork_intrinsics_dir)


def find_system_headers_path(config: dict) -> str:
    """
    Note: Only runs under Windows targets

    Tries to find the system headers included with the Mingw installation.
    Currently, using Zork with Clang under Windows depends of having a installation
    of GCC Gnu's compiler through MinGW.
    """
    SYSTEM_HEADERS_PATH: str = ''

    # Check if the user has a configured value for the system headers
    user_sys_headers: str = config.get('compiler').system_headers_path

    if user_sys_headers.__eq__(''):
        if constants.OS == constants.WINDOWS:
            # TODO Offer by default also the lookage for the MSVC toolchain
            # system headers?
            path = SYSTEM_HEADERS_EXPECTED_PATHS[0]
            gcc_version_folder = sorted(os.listdir(path), reverse=True)
            if len(gcc_version_folder) > 0:
                SYSTEM_HEADERS_PATH = path + gcc_version_folder[0]
    else:
        SYSTEM_HEADERS_PATH = user_sys_headers

    if SYSTEM_HEADERS_PATH != '':
        return SYSTEM_HEADERS_PATH
    raise NoSystemHeadersFound()


def generate_import_std(config: dict, zork_intrinsics_dir_path: str):
    """ Generates a zork.modulemap file that will be used to map all the
        system headers into just one module `std`, that also will acomplish
        the C++23 proposal of unifiying the standard library into only one
        module, bring it into scope as `import std;`
    """
    # TODO Check for rebuild argument
    if os.path.exists(f'{config.get("build").output_dir}/zork/intrinsics/std.h'):
        print("Cached out/zork directory found")
        return

    # If there's no user manually setted path, we perform the autosearch
    # for the system headers
    if config.get('compiler').system_headers_path == '':
        config['compiler'].system_headers_path = find_system_headers_path(config)
    SYS_HEADERS_BASE_PATH: str = config.get('compiler').system_headers_path

    # We are going to store a relation between the files inside of the root
    # of the folder of the include files, and the ones stored in a subdirectory.
    system_headers: dict = {}
    discarded_headers: list[str] =  ['cstdlib', 'stdlib.h', 'stacktrace']

    for root, _, sys_headers in os.walk(SYS_HEADERS_BASE_PATH):
        if root == SYS_HEADERS_BASE_PATH:
            system_headers.update({'root': sys_headers})
        else:
            pass
            # system_headers.update({root.removeprefix(SYS_HEADERS_BASE_PATH): sys_headers})
            # TODO study case, multiple headers causes redefinition problems when exported on the
            # module map file. Most std important ones even tho are reexported on the module map
    # print(f'sys headers: {system_headers}')

    # The files that will make possible the `import std;`
    ZORK_MODULE_MAP: str = ''
    SYSTEM_HEADERS_HEADER: str = ''  # All the #include <system_header> needed

    for path, sys_headers in system_headers.items():
        path = path.replace('\\', '/')
        for file in sys_headers:
            # print(f'FILE: {file}')
            if file.endswith('.tcc') or '.' in file or file in discarded_headers:
                continue
            if path == 'root':
                SYSTEM_HEADERS_HEADER += f'#include <{file}>\n'
            else:
                SYSTEM_HEADERS_HEADER += f'#include <{path}/{file}>\n'


    ZORK_MODULE_MAP += (
        'module "std"' + ' {\n'
        '  export *\n'
        '  header "std.h"\n'
        '}'
    )

    with open(f'{zork_intrinsics_dir_path}/std.h', 'w', encoding='UTF-8') as std_headers_file:
        std_headers_file.write(SYSTEM_HEADERS_HEADER)
    with open(f'{zork_intrinsics_dir_path}/zork.modulemap', 'w', encoding='UTF-8') \
        as zork_modulemap_file:
        zork_modulemap_file.write(ZORK_MODULE_MAP)


def run_subprocess(res: int) -> int:
    """ Parses the return code after calling a subprocess event """
    if res != 0:
        sys.exit()
    return res
