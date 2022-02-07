from program_definitions import COMPILER_ATTR, LANGUAGE_ATTR, BUILD_ATTR

""" Constant definitions across the whole program """
PROJECT_VERSION: str = '0.1.0'
CONFIGURATION_FILE_NAME: str = 'zork.conf'

# Suported compilers
CLANG: str = 'clang++'
GCC: str = 'g++'
MSVC: str = 'msbuild'
SUPPORTED_COMPILERS: list = [CLANG, GCC, MSVC]

""" Sections """
PROGRAM_SECTIONS: list = [
    COMPILER_ATTR, LANGUAGE_ATTR, BUILD_ATTR
]

# Shortcut to have all the attributes as identifiers
PROGRAM_ATTRIBUTES_IDENTIFIERS = [
    attr.identifier for attr in PROGRAM_SECTIONS
]
