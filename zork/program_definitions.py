"""[summary]

    This file provides the instances of the dataclasses with the program
    defined values for every section (attributes + properties)
    availables in Zork.
"""

from typing import Any

from data.attributes import CompilerAttribute, LanguageAttribute, \
    BuildAttribute, ExecutableAttribute, ProjectAttribute
from data.properties import Property
from data.user_config import CompilerConfig, ExecutableConfig, \
    LanguageConfig, BuildConfig, ModulesConfig, ProjectConfig

# Suported compilers
CLANG: str = 'clang++'
GCC: str = 'g++'
MSVC: str = 'msbuild'
SUPPORTED_COMPILERS: list = [CLANG, GCC, MSVC]

SUPPORTED_CPP_LANG_LEVELS: list = [
    '11', '14', '17', '20', '1a', '2a', '1x', '2x'
]
SUPPORTED_CPP_STDLIBS: list = ['stdlibc++', 'libc++']
SYSTEM_HEADERS_EXPECTED_PATHS: list[str] = ['C:/msys64/mingw64/include/c++/']


""" Zork Sections """
PROJECT_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#project]]',
    mandatory=True,
    properties=[
        Property('name', True, Any),
        Property('authors', False, Any)
    ]
)

COMPILER_ATTR: CompilerAttribute = CompilerAttribute(
    identifier='[[#compiler]]',
    mandatory=True,
    properties=[
        Property('cpp_compiler', True, SUPPORTED_COMPILERS),
        Property('system_headers_path', False, Any)
    ]
)

LANGUAGE_ATTR: LanguageAttribute = LanguageAttribute(
    identifier='[[#language]]',
    mandatory=True,
    properties=[
        Property(
            'cpp_standard', True, SUPPORTED_CPP_LANG_LEVELS
        ),
        Property(
            'std_lib', False, SUPPORTED_CPP_STDLIBS
        ),
        Property(
            'modules', False, 'true'
        ),
    ]
)

BUILD_ATTR: BuildAttribute = BuildAttribute(
    identifier='[[#build]]',
    mandatory=False,
    properties=[
        Property('output_dir', False, Any)
    ]
)

MODULES_ATTR: Property = BuildAttribute(
    identifier='[[#modules]]',
    mandatory=False,
    properties=[
        Property('base_ifcs_dir', False, Any),
        Property('interfaces', False, Any),
        Property('base_impls_dir', False, Any),
        Property('implementations', False, Any)
    ]
)

EXECUTABLE_ATTR: ExecutableAttribute = ExecutableAttribute(
    identifier='[[#executable]]',
    mandatory=False,
    properties=[
        Property('executable_name', False, Any),
        Property('sources', False, Any),
        Property('auto_execute', False, ['true', 'false']),
    ]
)

# Shortcut to have all the sections available in Zork
PROGRAM_SECTIONS: list = [
    PROJECT_ATTR,
    COMPILER_ATTR,
    LANGUAGE_ATTR,
    BUILD_ATTR,
    MODULES_ATTR,
    EXECUTABLE_ATTR
]

# Shortcut to have all the attributes as identifiers
PROGRAM_ATTRIBUTES_IDENTIFIERS = [
    attr.identifier for attr in PROGRAM_SECTIONS
]


# Default base definitions for the project properfies
# TODO refactor this into generate new instance in found, not defaults
PROGRAM_BASE_CONFIG: dict = {
    'project': ProjectConfig('new_project', []),
    'compiler': CompilerConfig('clang', ''),
    'language': LanguageConfig(11, 'libstdc++', ''),
    'build': BuildConfig('./build'),
    'modules': ModulesConfig('.', [], '.', []),
    'executable': ExecutableConfig('main', '', 'false')
}
