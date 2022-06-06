from dataclasses import dataclass

""" [summary]

    Represents a property that it's part of an attribute.

    An attribute can have one or more properties, but it's properties
    are declared through the dataclasse below.

    The 'Property' class is designed in a generic way, so you can attach
    as many properties as you need to every existing attribute in Zork.
    in this file exists a dataclass that generates
    So, attributes are property agnostic, because all the properties are
    defined in the same way, having:

        - identifier: the property name
        - mandatory: a boolean indicating if the property should be
            a must a have (just if the attribute also exists)
        - values: a list that contains the value(s) retrieved for the
            property from the configuration file
"""


@dataclass
class Property:
    """
        Represents a property that it's part of any attribute in Zork.
    """
    identifier: str
    mandatory: bool
    values: list
