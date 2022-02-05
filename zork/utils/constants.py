from data.attributes import CompilerAttribute, LanguageAttribute, BuildAttribute
from data.properties import CompilerProperty, LanguageStandardProperty, \
    BuildOutputPathProperty

""" Constant definitions """
PROJECT_VERSION: str = '0.1.0'
CONFIGURATION_FILE_NAME: str = 'zork.conf'


COMPILER_ATTR: CompilerAttribute = CompilerAttribute(
    identifier= '[[#compiler]]', 
    mandatory= True,
    properties=[
        CompilerProperty('cpp_compiler', True)
    ]
)


LANGUAGE_ATTR: LanguageAttribute = LanguageAttribute(
    identifier= '[[#language]]', 
    mandatory= True,
    properties=[
        LanguageStandardProperty('cpp_standard', True),
        LanguageStandardProperty('std_lib', False),
    ]
)


BUILD_ATTR: BuildAttribute = BuildAttribute(
    identifier= '[[#build]]', 
    mandatory= False,
    properties=[
        BuildOutputPathProperty('output_dir', False)
    ]
)


PROGRAM_SECTIONS: list = [
    COMPILER_ATTR, LANGUAGE_ATTR, BUILD_ATTR
]



# TODO Move from here
attributes_found: list = []
mandatory_attributes_found: list = []
missed_mandatory_attributes: list = []

# Compiler
CLANG: str = 'clang++'
GCC: str = 'g++'
MSVC: str = 'msbuild'
SUPPORTED_COMPILERS: list = [CLANG, GCC, MSVC]
