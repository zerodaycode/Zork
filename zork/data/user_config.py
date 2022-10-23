""" [summary]
    Provides dataclasses to store the options selected by the
    user in the configuration file after parse it
"""

from dataclasses import dataclass
from typing import Any


@dataclass
class ProjectConfig:
    """ The user defined project configuration """
    name: str
    authors: list

    def set_property(self, property_name: str, value: Any):
        """ Sets the value(s) for the members of the class,
            given any value related by the method's parameter
            'property name'"""
        if property_name == 'name':
            self.name = value
        if property_name == 'authors':
            self.authors = get_authors(value)


@dataclass
class CompilerConfig:
    """ The user defined compiler configuration """
    cpp_compiler: str
    system_headers_path: str

    def set_property(self, property_name: str, value: Any):
        """ Sets the value(s) for the members of the class,
            given any value related by the method's parameter
            'property name' """
        if property_name == 'cpp_compiler':
            self.cpp_compiler = value
        elif property_name == 'system_headers_path':
            self.system_headers_path = value


@dataclass
class LanguageConfig:
    """ The user defined C++ language configuration """
    cpp_standard: int
    std_lib: str
    modules: bool

    def set_property(self, property_name: str, value: Any):
        """ Sets the value(s) for the members of the class,
            given any value related by the method's parameter
            'property name' """
        if property_name == 'cpp_standard':
            self.cpp_standard = value
        elif property_name == 'std_lib':
            self.std_lib = value
        elif property_name == 'modules':
            if value in ('true', 'True', ''):
                self.modules = True
            else:
                self.modules = False


@dataclass
class ModulesConfig:
    """ The user defined details to work with C++20 modules """
    base_ifcs_dir: str
    interfaces: list
    base_impls_dir: str
    implementations: list

    def set_property(self, property_name: str, value: Any):
        """ Sets the value(s) for the members of the class,
            given any value related by the method's parameter
            'property name' """
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
    """ The path where the compiler files will be placed """
    output_dir: str

    def set_property(self, property_name: str, value: Any):
        """ Sets the value(s) for the members of the class,
            given any value related by the method's parameter
            'property name'"""
        if property_name == 'output_dir':
            self.output_dir = value


@dataclass
class ExecutableConfig:
    """ The user defined configuration for produce an executable file """
    executable_name: str
    sources: list
    auto_execute: str

    def set_property(self, property_name: str, value: Any):
        """ Sets the value(s) for the members of the class,
            given any value related by the method's parameter
            'property name'"""
        if property_name == 'executable_name':
            self.executable_name = value
        elif property_name == 'sources':
            self.sources = get_sources(value)
        elif property_name == 'auto_execute':
            self.auto_execute = value


@dataclass
class TestsConfig:
    """ The user defined configuration to run tests"""
    tests_name: str
    sources: list
    auto_run_tests: str

    def set_property(self, property_name: str, value: Any):
        """ Sets the value(s) for the members of the class,
            given any value related by the method's parameter
            'property name'"""
        if property_name == 'tests_name':
            self.tests_name = value
        elif property_name == 'sources':
            self.sources = get_sources(value)
        elif property_name == 'auto_run_tests':
            self.auto_run_tests = value


def get_authors(value) -> list:
    """ Retrieves the authors property for the project attribute
        as a list of str """
    sources = []
    for source in value.split(','):
        # Remove unnecesary whitespaces
        source = source.strip(' ')
        sources.append(source)
    return sources


def get_sources(value) -> list:
    """ Convenient function designed to retrieve the user defined
        source files """
    sources = []
    for source in value.split(','):
        # Remove unnecesary whitespaces
        source = source.strip(' ')
        sources.append(source)

    return sources


def get_interfaces(value: str) -> list:
    """ Convenient function designed to retrieve the user defined
        module interface units file names """
    sources = []
    if ';' in value:
        if value.endswith(';'):
            value = value[:-1]
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
                implementations: math.cpp=[math];math2.cpp=[math, math2]
                ...

            Code below could be read as: The math.cpp and math2.cpp module
            implementation units depends on the module interface math, and
            also math2.cpp depends on the module math2.

            - When the implementation file, has the same name of the
            module interface unit, Zork will automatically link them
            by passing the same name to the .pcm module.
    """
    sources = []
    for source in value.split(';'):
        # Removes trailing semicolons, that will produce an invalid iteration
        # over a non-file data (split.(';') will produce a relation that has
        # a path but not a file).
        if value.endswith(';'):
            value = value[:-1]
        # Remove unnecesary whitespaces
        source = source.strip(' ')
        sources.append(source)

    # TODO Even the impl is the same than the 'get_authors()', this
    # function possible changes in the future, so will stay as a
    # separated functions

    return sources
