"""[summary]

    This file provides the instances of the dataclasses with the program
    defined values for every section (attributes + properties) 
    availables in Zork.
"""

from data.attributes import CompilerAttribute, LanguageAttribute, BuildAttribute
from data.properties import CompilerProperty, LanguageStandardProperty, \
    BuildOutputPathProperty

COMPILER_ATTR: CompilerAttribute = CompilerAttribute(
    identifier = '[[#compiler]]', 
    mandatory = True,
    properties = [
        CompilerProperty('cpp_compiler', True)
    ]
)

LANGUAGE_ATTR: LanguageAttribute = LanguageAttribute(
    identifier = '[[#language]]', 
    mandatory = True,
    properties = [
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