""" Constant definitions """
PROJECT_VERSION: str = '0.1.0'
CONFIGURATION_FILE_NAME: str = 'zork.conf'

# Available attributes for configuring the project
COMPILER_ATTR: str = '[[#compiler]]'
LANGUAGE_ATTR: str = '[[#language]]'
BUILD_ATTR: str = '[[#build]]'

MANDATORY_ATTRIBUTES: list = [COMPILER_ATTR, LANGUAGE_ATTR]

attributes_found: list = []
mandatory_attributes_found: list = []
missed_mandatory_attributes: list = []

# Compiler
CLANG: str = 'clang++'
GCC: str = 'g++'
MSVC: str = 'msbuild'
SUPPORTED_COMPILERS: list = [CLANG, GCC, MSVC]
