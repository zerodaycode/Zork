"""[summary]

    This file provides the instances of the dataclasses with the program
    defined values for every section (attributes + properties)
    availables in Zork.
"""

from typing import Any

from data.attributes import ProjectAttribute
from data.properties import Property

from data.user_config import ProjectConfig, CompilerConfig, LanguageConfig, \
    ModulesConfig, BuildConfig, ExecutableConfig, TestsConfig

# Suported compilers
CLANG: str = 'clang++'
GCC: str = 'g++'
MSVC: str = 'msbuild'
SUPPORTED_COMPILERS: list = [CLANG, GCC, MSVC]

SUPPORTED_CPP_LANG_LEVELS: list = [
    '11', '14', '17', '20', '23', '1a', '2a', '1x', '2x'
]
SUPPORTED_CPP_STDLIBS: list[str] = ['libstdc++', 'libc++']
SYSTEM_HEADERS_EXPECTED_PATHS: list[str] = ['C:/msys64/mingw64/include/c++/']


""" Zork Sections """
PROJECT_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#project]]',
    mandatory=True,
    properties=[
        Property(
            identifier='name',
            mandatory=True,
            values=Any
        ),
        Property(
            identifier='authors',
            mandatory=False,
            values=Any
        )
    ]
)

COMPILER_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#compiler]]',
    mandatory=True,
    properties=[
        Property(
            identifier='cpp_compiler',
            mandatory=True,
            values=SUPPORTED_COMPILERS
        ),
        Property(
            identifier='extra_args',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='system_headers_path',
            mandatory=False,
            values=Any
        )
    ]
)

LANGUAGE_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#language]]',
    mandatory=True,
    properties=[
        Property(
            identifier='cpp_standard',
            mandatory=True,
            values=SUPPORTED_CPP_LANG_LEVELS
        ),
        Property(
            identifier='std_lib',
            mandatory=False,
            values=SUPPORTED_CPP_STDLIBS
        ),
        Property(
            identifier='modules',
            mandatory=False,
            values=['True', 'true']
        ),
    ]
)

BUILD_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#build]]',
    mandatory=False,
    properties=[
        Property(
            identifier='output_dir',
            mandatory=False,
            values=Any
        )
    ]
)

EXECUTABLE_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#executable]]',
    mandatory=False,
    properties=[
        Property(
            identifier='executable_name',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='sources_base_path',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='sources',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='auto_execute',
            mandatory=False,
            values=['true', 'false']
        ),
        Property(
            identifier='extra_args',
            mandatory=False,
            values=Any
        ),
    ]
)

MODULES_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#modules]]',
    mandatory=False,
    properties=[
        Property(
            identifier='base_ifcs_dir',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='interfaces',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='base_impls_dir',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='implementations',
            mandatory=False,
            values=Any
        )
    ]
)

TESTS_ATTR: ProjectAttribute = ProjectAttribute(
    identifier='[[#tests]]',
    mandatory=False,
    properties=[
        Property(
            identifier='tests_executable_name',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='sources_base_path',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='sources',
            mandatory=False,
            values=Any
        ),
        Property(
            identifier='auto_run_tests',
            mandatory=False,
            values=['true', 'false']
        ),
        Property(
            identifier='extra_args',
            mandatory=False,
            values=Any
        ),
    ]
)


# Shortcut to have all the sections available in Zork
PROGRAM_SECTIONS: list = [
    PROJECT_ATTR,
    COMPILER_ATTR,
    LANGUAGE_ATTR,
    BUILD_ATTR,
    MODULES_ATTR,
    EXECUTABLE_ATTR,
    TESTS_ATTR
]

# Shortcut to have all the attributes as identifiers
PROGRAM_ATTRIBUTES_IDENTIFIERS = [
    attr.identifier for attr in PROGRAM_SECTIONS
]


# Default base definitions for the project properfies
# TODO refactor this into generate new instance in found, not defaults
PROGRAM_BASE_CONFIG: dict = {
    'project': ProjectConfig('new_project', []),
    'compiler': CompilerConfig('clang', [], ''),
    'language': LanguageConfig(11, 'libstdc++', True),
    'build': BuildConfig('./build'),
    'modules': ModulesConfig('.', [], '.', []),
    'executable': ExecutableConfig('main', '', '', 'false', []),
    'tests': TestsConfig('proj_tests', '', '', 'false', [])
}
