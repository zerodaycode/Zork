from dataclasses import dataclass

""" [summary]

    Every dataclass below represents a property that it's being part
    of an attribute.

    An attribute can have one or more properties, but it's properties
    are declared through this dataclasses.

    This dataclasses are designed in a generic way, so for every
    attribute in Zork, in this file exists a dataclass that generates
    the property/properties that the attribute can hold. So, they
    are property agnostic, because all the properties are defined in the 
    same way, having:

        - identifier: the property name
        - mandatory: a boolean indicating if the property should be
            a must a have (just if the attribute also exists)
        - values: a list that contains the value(s) retrieved for the
            property from the configuration file
"""


@dataclass
class CompilerProperty:
    """
        A property that it's part of the Compiler attribute.
        Represents the compilers available by Zork.
    """
    identifier: str
    mandatory: bool
    values: list


@dataclass
class LanguageStandardProperty:
    """
        A property that it's part of the Compiler attribute.

        Represents a value over a C++ posible configuration,
        like the language standard level, the std library to use,
        if the modules feature (> C++20 language level) it's present,
        and you are building a project with modules, instead the classical
        headers...
    """
    identifier: str
    mandatory: bool
    values: list


@dataclass
class BuildProperty:
    """
        A property that it's part of the Build attribute.
        Properties for control where the compiler's output will be placed.
    """
    identifier: str
    mandatory: bool
    values: list


@dataclass
class ExecutableProperty:
    """
        A property that it's part of the Executable attribute.
        Definitions for the available options for building an executable
    """
    identifier: str
    mandatory: bool
    values: list
