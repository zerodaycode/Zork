import typing
import re

from utils.exceptions import AttributeDependsOnProperty, DuplicatedAttribute,\
    MissedMandatoryAttributes, UnknownAttribute,\
    UnknownProperties, ErrorFileFormat,\
    MissedMandatoryProperties, InvalidPropertyValue
from utils.constants import CONFIGURATION_FILE_NAME
from utils.regex_patterns import VALID_LINE_PATTERN, RE_VALID_LINE_FORMAT

from program_definitions import PROGRAM_BASE_CONFIG, \
    PROGRAM_ATTRIBUTES_IDENTIFIERS, PROGRAM_SECTIONS


def get_project_config(root_path: str, verbose) -> dict:
    """Parses the file looking for a kind of AST token tokens"""

    # Open the configuration file in 'read-only' mode
    config_file = read_config_file_lines(root_path)

    # Check if the config file format it's valid
    check_valid_config_file(config_file)

    # If the config file it's OK, the we can retrieve all the config sections
    return get_sections(config_file, verbose)


def read_config_file_lines(root_path: str) -> list:
    """ Get all the lines written in the conf file """
    with open(root_path + '/' + CONFIGURATION_FILE_NAME, 'r') as config_file:
        return config_file.readlines()


def check_valid_config_file(config_file: list):
    """ Ensures that the content written in the config file
        it's valid for the Zork config language """
    for idx, line in enumerate(config_file):
        line = line.strip()

        if line and not re.match(RE_VALID_LINE_FORMAT, line):
            raise ErrorFileFormat(idx + 1, line)


def clean_file(file: str) -> list:
    """
        Cleans the file and retrieves only lines with attributes or properties
    """
    return re.findall(
        VALID_LINE_PATTERN, file, re.MULTILINE
    )


def parse_attr_properties_block(file: str) -> dict:
    """ Gets every syntactically valid attribute with the founded properties,
        discards the unknown attributes
    """
    block_pattern = r"^\[\[#\w+]]\n(?:^\w+: ?.+\n?)+"
    blocks = re.findall(block_pattern, file, re.MULTILINE)

    retrieved_data = {}

    for block in blocks:
        attr_pattern = r"^\[\[#(\w+)]]"
        property_pattern = r"^(.+): (.+)$"

        attribute_identifier = re.search(attr_pattern, block).group(0)
        # Check for attributes that dont' belong to the program designed ones
        if attribute_identifier not in PROGRAM_ATTRIBUTES_IDENTIFIERS:
            raise UnknownAttribute(attribute_identifier)

        extracted_properties = re.findall(
            property_pattern, block, re.MULTILINE
        )

        properties: list = []
        for property_name, property_value in extracted_properties:
            properties.append(
                {
                    "property_name": property_name,
                    "property_value": property_value
                }
            )

        retrieved_data[attribute_identifier] = properties

    return retrieved_data


def validate_special_cases(config_file_sections: dict):
    """_summary_

        Validates special cases, like attributes that only must be present
        if there's another attribute or another property attribute
        that enables it
    """
    # Case that if the [[#modules]] attribute is present but in
    # the [[#language]] attribute, the value of the property 'modules'
    # is not equuals to 'true'
    if config_file_sections.__contains__('[[#modules]]'):
        for ppty in config_file_sections.get('[[#language]]'):
            if ppty.get('property_name') == 'modules':
                if ppty.get('property_value') != 'true':
                    raise AttributeDependsOnProperty(
                        '[[#modules]]',
                        '[[#language]]',
                        'modules',
                        'true'
                    )


def get_sections(config_file: str, verbose: bool) -> dict:
    """ Recovers the sections described in the config file, returning a dict with
        the instances of the dataclasses designed for carry the final data
        to the compiler """

    # Initializes the map with the config values and provide default values
    config: dict = PROGRAM_BASE_CONFIG

    cleaned_config_file: list = clean_file("".join(config_file))
    attr_ppt_collection = parse_attr_properties_block(
        '\n'.join(cleaned_config_file)
    )

    validate_special_cases(attr_ppt_collection)
    print(f'SECTIONS: {attr_ppt_collection}')

    """
        Once we have parsed and cleaned the sections founded on the
        config file, we can start match them against the valid ones
        (the ones allowed by Zork).
        Until here, we only validated that the code written on the
        conf file it's syntanctically correct acording the rules
        provided by the program.
        Now we must discover if the retrived data it's also available
        and exists inside Zork.
    """

    # Tracks the mandatory attributes not written in the config file
    founded_attributes: list = []
    missed_mandatory_attributes: list = []

    # Check for duplicated attributes
    for attribute, _ in attr_ppt_collection.items():
        if attribute not in founded_attributes:
            founded_attributes.append(attribute)
        else:
            raise DuplicatedAttribute(attribute)
    if verbose:
        print(f'\nFounded attributes on the config file: {founded_attributes}')

    for section in PROGRAM_SECTIONS:
        # Try to get the same section (if exists in the config file)
        # In this way, we can also check if all the mandatory attributes are
        # configured, and are valid ones
        config_file_section_properties = attr_ppt_collection.get(
            section.identifier
        )

        # The logic for a valid founded property goes here
        if config_file_section_properties is not None:
            if verbose:
                print(f'\tFinded attribute: {section.identifier}')
                print(f'\tFinded properties: {config_file_section_properties}')
            parse_properties_for_current_attribute(
                section, config_file_section_properties, config, verbose
            )
            if verbose:
                print('')
        else:
            if section.mandatory is True:
                missed_mandatory_attributes.append(section.identifier)

    if len(missed_mandatory_attributes) > 0:
        raise MissedMandatoryAttributes(missed_mandatory_attributes)

    return config


def parse_properties_for_current_attribute(
    section, config_file_section_properties, config, verbose
):
    """ Parses and validates the properties found for a given attribute """
    detected_properties_for_current_attribute = [
        ppt_identifier['property_name'] for ppt_identifier in
        config_file_section_properties
    ]

    # Check for mandatory properties for the current attribute
    check_for_mandatory_properties(
        section,
        detected_properties_for_current_attribute
    )

    # If we have all the mandatory ones, unpack the founded properties to
    # validate them
    validate_founded_properties(
        section,
        detected_properties_for_current_attribute,
        config_file_section_properties,
        verbose
    )

    """
        Validation it's made through several function calls that performs
        runtime checks to the retrieved values, matching them against
        the valid defined ones on this program using an exception-flow-control
        based style.
        So if there's no exceptions raised until here, we can safetly
        retrieve the values founded on the config file.

        By the way, if an exception it's raised in the process,
        the program will exit, logging the exception stack that
        triggered the exception event
    """

    # If everything it's valid, we can fill our config dict with the data
    for validated_property in config_file_section_properties:
        # CARE. We are modifying by reference the config dict
        config[section.identifier[3:-2]].set_property(
            validated_property['property_name'],
            validated_property['property_value']
        )
    """
        This means that the keys of the config dict are strings with a
        pre-defined value that represents the same value as the
        self.identifier property but without the Zork syntantic
        elements to define an attribute, ie -> [[#...]]

        EX:
            config: dict = {
                'compiler' : CompilerConfig('clang'),
                'language' : LanguageConfig(20, 'libstdc++'),
                'build' : BuildConfig('./build')
            }

        where the 'compiler' key matches the self.identifier = [[#compiler]]
        class attribute of the instance stored as a value of that key

        config['compiler'] = class CompilerConfig
        config['compiler'].identifier = '[[#compiler]'
        config['compiler'][3:-2] =
            config[section.identifier[3:-2]] = 'compiler'
    """


def check_for_mandatory_properties(
    section, detected_properties_for_current_attribute
):
    """ Checks if all of the mandatory properties for the current attribute
        are written in the section """
    missed_mandatory_properties: list = []

    for program_property in section.properties:
        if program_property.mandatory is True:
            if program_property.identifier not in \
                    detected_properties_for_current_attribute:
                missed_mandatory_properties.append(program_property.identifier)

    if len(missed_mandatory_properties) > 0:
        raise MissedMandatoryProperties(
            missed_mandatory_properties, section.identifier
        )


def validate_founded_properties(
    section,
    detected_properties_for_current_attribute,
    config_file_section_properties,
    verbose
):
    """ Validates the identifier of a given property """
    invalid_properties_found: list = []

    # List with the program defined property identifiers
    # for the current attribute
    program_defined_property_identifiers_for_current_attribute = [
        property.identifier for property in section.properties
    ]

    for elem_idx, ppt_identifier in enumerate(
        detected_properties_for_current_attribute
    ):
        if verbose:
            print(f'\tLooking for: {ppt_identifier} property')
        # Raises exception if the property isn't allowed on Zork
        if ppt_identifier not in \
                program_defined_property_identifiers_for_current_attribute:
            invalid_properties_found.append(ppt_identifier)
        else:  # Check if the founded property also has a valid value
            # Retrieve the property value from the config file
            ppt_value = \
                config_file_section_properties[elem_idx]['property_value']

            if verbose:
                print(
                    f'\tGetting: {ppt_value} as value, ' +
                    f'with type: {type(ppt_value)}'
                )

            # Retrieve the allowed values by Zork for a given property
            allowed_property_values = [
                property.values for property in section.properties
                if property.identifier == ppt_identifier
            ][0]
            if verbose:
                print(
                    f'\tAllowed value(s) {allowed_property_values} with type:'
                    + f'{ type(allowed_property_values)} for property: ' +
                    f'{ppt_identifier}'
                )

            # typing._SpecialForm is the type associated with the values
            # property of any Property that the values property
            # type == typing.Any
            if type(allowed_property_values) != typing._SpecialForm and \
                    ppt_value not in allowed_property_values:
                raise InvalidPropertyValue(ppt_value, ppt_identifier)

    if len(invalid_properties_found) > 0:
        raise UnknownProperties(invalid_properties_found, section.identifier)
