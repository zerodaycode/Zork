import glob

from dataclasses import dataclass
from typing import Any

"""[summary]
    Provides dataclasses to store the options selected by the
    user in the configuration file after parse it
"""


@dataclass
class CompilerConfig:
    cpp_compiler: str

    def set_property(self, property_name: str, value: Any):
        if property_name == 'cpp_compiler':
            self.cpp_compiler = value


@dataclass
class LanguageConfig:
    cpp_standard: int
    std_lib: str
    modules: bool

    def set_property(self, property_name: str, value: Any):
        if property_name == 'cpp_standard':
            self.cpp_standard = value
        elif property_name == 'std_lib':
            self.std_lib = value
        elif property_name == 'modules':
            self.modules = value


@dataclass
class ModulesConfig:
    interfaces_dirs: str
    interfaces: list
    implementations_dirs: str
    implementations: list

    def set_property(self, property_name: str, value: Any):
        if property_name == 'interfaces_dirs':
            self.interfaces_dirs = get_dirs(value)
        elif property_name == 'interfaces':
            self.interfaces = get_sources(value)
        elif property_name == 'implementations_dirs':
            self.implementations_dirs = get_dirs(value)
        elif property_name == 'implementations':
            self.implementations = get_sources(value)


@dataclass
class BuildConfig:
    output_dir: str

    def set_property(self, property_name: str, value: Any):
        if property_name == 'output_dir':
            self.output_dir = value


@dataclass
class ExecutableConfig:
    executable_name: str
    sources: list
    auto_execute: str

    def set_property(self, property_name: str, value: Any):
        if property_name == 'executable_name':
            self.executable_name = value
        elif property_name == 'sources':
            self.sources = get_sources(value)
        elif property_name == 'auto_execute':
            self.auto_execute = value


def get_dirs(value) -> list:
    """ Convenient function designed to retrieve the user defined
        paths for modules as a list"""
    sources = []
    for source in value.split(','):
        # Remove unnecesary whitespaces
        source = source.strip(' ')
        # Check if it's a path, add the relative ./ to the Zork config file
        if source.__contains__('/') and not source.startswith('./'):
            source = './' + source

        sources.append(source)
    return sources


def get_sources(value) -> list:
    """ Convenient function designed to retrieve the user defined
        source files or module units file names """
    sources = []
    for source in value.split(','):
        # Remove unnecesary whitespaces
        source = source.strip(' ')
        # Check if it's a path, add the relative ./ to the Zork config file
        if source.__contains__('/') and not source.startswith('./'):
            source = './' + source
        # Check for wildcards, so every file in the provided directory
        # should be included
        if source.__contains__('*') and not source.startswith('**'):
            for wildcarded_source in glob.glob(source):
                sources.append(wildcarded_source)
        else:
            sources.append(source)
    return sources


def generate_mod_impl_units(value) -> list:
    """ Generates the module implementation units in a way that
        Zork can understand.

        This is due to the fact that when the definition it's splitted
        from the implementation, the compilation of the module implementation
        units must be 'linked' with the interface. In Clang, this
        is made it by pass the parameter '-fmodule-file=<value>',
        where the value must point to a precompiled module unit.

        To make the relation, the source files declared on the Zork
        config file on the [[#modules]] attribute -> 'implementations'
        property, can be related like:

            - When the implementation file name does not match the same
            name of the module interface unit, or you have one that matches
            but other implementation files that do not, Zork will need to
            know to what module interface unit must link against. So, a special
            syntax it's created in the Zork config file for this case.

            This is just by creating a tuple with the relative path
            of the module implementation unit, and the file name of
            the module interface unit, without extension (it's already)
            precompiled, and Zork already knows where to find it.

            It will looks like:
                ...
                implementations: (*/math.cpp, math)
                ...

            Code below could be read as: Take the math.cpp file and
            link it against the precompiled module math.

            - WWhen the implementation file, has the same name of the
            module interface unit, Zork will automatically link them
            by passing the same name to the .pcm module.
    """
    sources = []
    for source in value.split(','):
        # Remove unnecesary whitespaces
        source = source.strip(' ')
        # Check if it's a path, add the relative ./ to the Zork config file
        if source.__contains__('/') and not source.startswith('./'):
            source = './' + source
        # Check for wildcards, so every file in the provided directory
        # should be included
        if source.__contains__('*') and not source.startswith('**'):
            for wildcarded_source in glob.glob(source):
                sources.append(wildcarded_source)
        else:
            sources.append(source)
    return sources
