from dataclasses import dataclass

"""[summary]
    Classes for store constant data about the internal configuration
    (elected by design) of the program attributes and properties
"""


@dataclass
class CompilerProperty:
    """ Represents the compilers available by Zork """
    identifier: str
    mandatory: bool
    values: list


@dataclass
class LanguageStandardProperty:
    """ Sets the C++ language standard passed to the compiler  """
    identifier: str
    mandatory: bool
    values: list


@dataclass
class BuildOutputPathProperty:
    """ The place where the compiler's output will be placed """
    identifier: str
    mandatory: bool
    values: list


@dataclass
class ExecutableProperty:
    """ Definitions for the available options for building an executable """
    identifier: str
    mandatory: bool
    values: list
