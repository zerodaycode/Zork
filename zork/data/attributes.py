"""[summary]
    Classes for store constant data about the internal configuration
    (elected by design) of the program attributes and properties
"""

from dataclasses import dataclass


@dataclass
class ProjectAttribute:
    """ Represents an attribute on Zork """
    identifier: str
    mandatory: bool
    properties: list
