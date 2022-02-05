from dataclasses import dataclass

"""[summary]
    Classes for store constant data about the internal configuration 
    (elected by design) of the program attributes and properties
""" 

@dataclass
class CompilerProperty:
    """ Represents the structure of the compiler property """
    identifier: str
    mandatory: bool

    def as_dict(self) -> dict:
        return {
            'identifier': self.identifier, 'mandatory': self.mandatory
        }

@dataclass
class LanguageStandardProperty:
    """ Represents the structure of the compiler property """
    identifier: str
    mandatory: bool

    def as_dict(self) -> dict:
        return {
            'identifier': self.identifier, 'mandatory': self.mandatory
        }

@dataclass
class BuildOutputPathProperty:
    """ Represents the structure of the compiler property """
    identifier: str
    mandatory: bool

    def as_dict(self) -> dict:
        return {
            'identifier': self.identifier, 'mandatory': self.mandatory
        }