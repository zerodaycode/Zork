"""[summary]
    Classes for store constant data about the internal configuration
    (elected by design) of the program attributes and properties
"""

from dataclasses import dataclass


@dataclass
class ProjectAttribute:
    """ Holds information about the project itself """
    identifier: str
    mandatory: bool
    properties: list


@dataclass
class CompilerAttribute:
    """ Represents the structure of the compiler attribute """
    identifier: str
    mandatory: bool
    properties: list


@dataclass
class LanguageAttribute:
    """ Represents the structure of the language property """
    identifier: str
    mandatory: bool
    properties: list


@dataclass
class BuildAttribute:
    """ Represents the structure of the build property """
    identifier: str
    mandatory: bool
    properties: list


@dataclass
class ModulesAttribute:
    """ Represents the structure of the build property """
    identifier: str
    mandatory: bool
    properties: list


@dataclass
class ExecutableAttribute:
    """ Holds the configuration for generate an executable """
    identifier: str
    mandatory: bool
    properties: list
