from dataclasses import dataclass
from typing import Any

"""[summary]
    Provides dataclasses to store the options selected by the
    user in the configuration file after parse it
"""


@dataclass
class ProjectConfig:
    name: str
    authors: list

    def set_property(self, property_name: str, value: Any):
        if property_name == 'name':
            self.name = value
        if property_name == 'authors':
            self.authors = get_authors(value)


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
    base_ifcs_dir: str
    interfaces: list
    base_impls_dir: str
    implementations: list

    def set_property(self, property_name: str, value: Any):
        if property_name == 'base_ifcs_dir':
            self.base_ifcs_dir = value
        elif property_name == 'interfaces':
            self.interfaces = get_interfaces(value)
        elif property_name == 'base_impls_dir':
            self.base_impls_dir = value
        elif property_name == 'implementations':
            self.implementations = generate_mod_impl_units(value)


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


def get_authors(value) -> list:
    """ CRetrieves the authors property for the project attribute
        as a list of str
    """
    sources = []
    for source in value.split(','):
        # Remove unnecesary whitespaces
        source = source.strip(' ')
        sources.append(source)
    return sources


def get_sources(value) -> list:
    """ Convenient function designed to retrieve the user defined
        source files"""
    sources = []
    for source in value.split(','):
        # Remove unnecesary whitespaces
        source = source.strip(' ')
        sources.append(source)

    return sources


def get_interfaces(value) -> list:
    """ Convenient function designed to retrieve the user defined
        module interface units file names """
    sources = []
    if ';' in value:
        for source in value.split(';'):
            source = source.strip(' ')
            sources.append(source)
    else:
        for source in value.split(','):
            source = source.strip(' ')
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

            It will looks like:
                ...
                implementations: [math.cpp, math2.cpp]=math;
                    [math2.cpp, math22.cpp]=math2
                ...

            Code below could be read as: Take the math.cpp file and
            link it against the precompiled module math.

            - When the implementation file, has the same name of the
            module interface unit, Zork will automatically link them
            by passing the same name to the .pcm module.
    """
    sources = []
    for source in value.split(';'):
        # Remove unnecesary whitespaces
        source = source.strip(' ')
        sources.append(source)

    # TODO Even the impl is the same than the 'get_authors()', this
    # function possible changes in the future, so will stay as a
    # separated functions

    return sources
