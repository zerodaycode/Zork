"""[summary]

    This file provides the instances of the dataclasses with the program
    defined values for every section (attributes + properties) 
    availables in Zork.
"""

from typing import Any

from data.attributes import CompilerAttribute, LanguageAttribute, \
    BuildAttribute, ExecutableAttribute
from data.properties import CompilerProperty, LanguageStandardProperty, \
    BuildOutputPathProperty, ExecutableProperty

from data.structures import CompilerConfig, ExecutableConfig, LanguageConfig, \
    BuildConfig

# Suported compilers
CLANG: str = 'clang++'
GCC: str = 'g++'
MSVC: str = 'msbuild'
SUPPORTED_COMPILERS: list = [CLANG, GCC, MSVC]

SUPPORTED_CPP_LANG_LEVELS: list = [
    '11', '14', '17', '20', '1a', '2a', '1x', '2x'
]
SUPPORTED_CPP_STDLIBS: list = ['stdlibc++', 'libc++']


""" Zork Sections """
COMPILER_ATTR: CompilerAttribute = CompilerAttribute(
    identifier='[[#compiler]]',
    mandatory=True,
    properties=[
        CompilerProperty('cpp_compiler', True, SUPPORTED_COMPILERS)
    ]
)

LANGUAGE_ATTR: LanguageAttribute = LanguageAttribute(
    identifier='[[#language]]',
    mandatory=True,
    properties=[
        LanguageStandardProperty(
            'cpp_standard', True, SUPPORTED_CPP_LANG_LEVELS
        ),
        LanguageStandardProperty(
            'std_lib', False, SUPPORTED_CPP_STDLIBS
        ),
    ]
)

BUILD_ATTR: BuildAttribute = BuildAttribute(
    identifier='[[#build]]',
    mandatory=False,
    properties=[
        BuildOutputPathProperty('output_dir', False, Any)
    ]
)

EXECUTABLE_ATTR: ExecutableAttribute = ExecutableAttribute(
    identifier='[[#executable]]',
    mandatory=False,
    properties=[
        ExecutableProperty('executable_name', False, Any),
        ExecutableProperty('sources', False, Any),
        ExecutableProperty('auto_execute', False, ['true', 'false']),
    ]
)


PROGRAM_SECTIONS: list = [
    COMPILER_ATTR, LANGUAGE_ATTR, BUILD_ATTR, EXECUTABLE_ATTR
]

# Shortcut to have all the attributes as identifiers
PROGRAM_ATTRIBUTES_IDENTIFIERS = [
    attr.identifier for attr in PROGRAM_SECTIONS
]


PROGRAM_BASE_CONFIG: dict = {
    'compiler': CompilerConfig('clang'),
    'language': LanguageConfig(20, 'libstdc++'),
    'output_dir': BuildConfig('./build'),
    'executable': ExecutableConfig('main', '', 'false')
}
