"""[summary]

    Provides custom exceptions for incorrect, unavailable or unsupported
    configuration on a Zork project
"""

from utils.constants import PROJECT_VERSION


class NoConfigurationFileFound(Exception):
    """
        Triggered when the program it's launched and no Zork config file
        is found
    """
    def __init__(self, message: str = 'No zork.conf file found'):
        self.message = message
        super().__init__(self.message)


class DuplicatedAttribute(Exception):
    """ If an attribute it's found by duplicated on the config file """
    def __init__(self, attr_name: str):
        super().__init__(f'{attr_name} is already defined in the file')


class MissedMandatoryAttributes(Exception):
    """ A mandatory attribute it's not defined in the configuration file """
    def __init__(self, missed_attrs: list):
        attr_str = 'Attribute: ' if len(missed_attrs) == 1 else 'Attributes: '
        is_are = "is" if len(missed_attrs) == 1 else "are"
        isnt_arent = "isn't" if len(missed_attrs) == 1 else "aren't"
        super().__init__(
            f'\n\t{attr_str + ", " .join(map(str, missed_attrs))}, ' +
            f'which {is_are} mandatory, {isnt_arent} present ' +
            'in the config file'
        )


class MissedMandatoryProperties(Exception):
    """ A mandatory Property it's not defined in the configuration file """
    def __init__(self, missed_ppts: list, section_identifier: str):
        attr_str = 'Property: ' if len(missed_ppts) == 1 else 'Properties: '
        is_are = "is" if len(missed_ppts) == 1 else "are"
        isnt_arent = "isn't" if len(missed_ppts) == 1 else "aren't"
        super().__init__(
            f'\n\t{attr_str + ", " .join(map(str, missed_ppts))}, which '
            f'{is_are} mandatory for the {section_identifier} attribute, ' +
            f'{isnt_arent} present in the config file'
        )


class UnknownAttribute(Exception):
    """ Not defined or available attribute found """
    def __init__(self, attr_name: str):
        super().__init__(f'{attr_name} is an unknown or unsupported attribute')


class UnknownProperty(Exception):
    """ Not defined or available attribute found " """
    def __init__(self, property_name: str, section_identifier: str):
        super().__init__(
            f'{property_name} is an unknown or unsupported property ' +
            f'for the {section_identifier} attribute'
        )


class InvalidPropertyValue(Exception):
    """ Not defined or available attribute found " """
    def __init__(self, property_value: str, property_name: str):
        super().__init__(
            f'<{property_value}> is an unknown or unsupported value ' +
            f'for the <{property_name}> property'
        )


class UnknownProperties(Exception):
    """
        A bulk with all the detected invalid properties written
        on the config file
    """
    def __init__(self, missed_ppts: list, section_identifier: str):
        attr_str = 'property' if len(missed_ppts) == 1 else 'properties:'
        is_are = "is" if len(missed_ppts) == 1 else "are"
        super().__init__(
            f'\n\tFound {attr_str} [{", " .join(map(str, missed_ppts))}] ' +
            f'which {is_are} unknown or invalid for the ' +
            f'{section_identifier} attribute'
        )


class ErrorFileFormat(Exception):
    """ Not defined or available attribute found " """
    def __init__(self, idx, error):
        super().__init__(
            f'ERROR in line: {idx}: \n\t{error}\n' +
            'Not valid sentence or format error'
        )


class UnsupportedCompiler(Exception):
    """ Not defined or available attribute found " """
    def __init__(self, compiler: str):
        super().__init__(
            f'<{compiler}> compiler it\'s unsupported at the actual ' +
            f'version of Zork v<{PROJECT_VERSION}>'
        )


class NoSystemHeadersFound(Exception):
    """ Not defined or available attribute found " """
    def __init__(self):
        super().__init__(
            "We could not found a include path for the system headers.\n" +
            "Currently, Zork can only work under Windows in automatic mode with Clang if a " +
            "mingw64 installation exists on the system.\n" +
            "Please, go and install a valid toolchain with MSYS2 to work with Clang, or " +
            "manually point to a Clang's installation in your system\n" +
            "that contains the C++ expected system headers using the " +
            "system_headers_path property " +
            "under the [[#compiler]] attribute"
        )

class LanguageLevelNotEnought(Exception):
    """ When a C++ feature it's requested, but the language level
        isn't higher enought that the feature does not exists for that
        standard.

        Ex: C++ modules features, requires at least, C++20.
        So, if C++17 it's selected as the language level by the
        user in the config file, this error w'd be raised.
    """
    def __init__(
        self,
        lang_level_required: int,
        lang_level_selected: int,
        feature: str
    ):
        super().__init__(
            f'C++ {feature} feature requires to set the language level to' +
            f', at least, C++{lang_level_required}. ' +
            f'Current is C++{lang_level_selected}.'
        )


class AttributeDependsOnProperty(Exception):
    """
       Raised when a certain Zork attribute it's found, but, for work,
       need to be enabled by a property on another attribute.

       Ex: The [[#modules]] attribute can only exists in the config
       file if previously in the [[#language]] attribute the value of
       the property 'modules' is equals to the string true.
    """
    def __init__(
        self,
        attribute: str,
        dependant_attribute: str,
        dependant_property: str,
        equals_to: str
    ):
        super().__init__(
            f'{attribute} needs that the property <{dependant_property}>' +
            f', that belongs to the {dependant_attribute} attribute, ' +
            f'to be equals to <{equals_to}>'
        )
