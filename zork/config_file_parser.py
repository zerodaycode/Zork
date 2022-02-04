from exceptions import DuplicatedAttribute, MissedMandatoryAttributes, \
    UnknownAttribute, UnknownProperty
from structures import CompilerConfig, LanguageConfig, BuildConfig
from constants import *
import re

# Initializes the map with the config values and provide default values
config: dict = {
    'compiler': CompilerConfig('clang'),
    'language': LanguageConfig(20, 'libstdc++'),
    'build': BuildConfig('./build')
}


def get_project_config(root_path: str) -> dict:
    """Parses the file looking for a kind of AST token tokens"""

    # Open the configuration file in 'read-only' mode
    config_file = read_config_file_lines(root_path)

    # Check mandatory tokens found
    check_mandatory_attributes()

    return config


def read_config_file_lines(root_path: str) -> list:
    with open(root_path + '/' + CONFIGURATION_FILE_NAME, 'r') as config_file:
        # Get all the lines written in the conf file
        return config_file.readlines()


def retrieve_attributes_identifier(file: str):
    pattern_get_attributes = r"^\[\[#\w+]]"
    attributes_found.append(re.findall(pattern_get_attributes, file, re.MULTILINE))


def check_mandatory_attributes():
    """ Checks if all the defined as 'mandatory attribute' elements
        are present in the configuration file
    """
    # Check if they are present. If not, append it to a tracking list
    for mandatory_attr in MANDATORY_ATTRIBUTES:
        if mandatory_attr in attributes_found:
            mandatory_attributes_found.append(mandatory_attr)
        else:
            missed_mandatory_attributes.append(mandatory_attr)
    # Raise an exception containing all the missed mandatory attributes
    if len(missed_mandatory_attributes) != 0:
        raise MissedMandatoryAttributes(missed_mandatory_attributes)


def precheck_valid_config_file(file: list):
    retrieve_attributes_identifier(file)
    if not attributes_found:
        pass
        # TODO Implement new exception NoAttributesFound
        # raise NoAttributesFound()
    check_mandatory_attributes()

    # Pattern to find if the file starts with an attribute or one or more comments before the attribute
    start_pattern = r"^(?:(?:# ?\w+\n)+\n?)*\[\[#\w+]]"

    if not start_pattern.match(file):
        pass
        # TODO Implement new exception ErrorStartFileFormat
        # raise ErrorStartFileFormat()


def clean_file(file: str) -> str:
    """Clean the file and retrieve only lines with attribute or property format"""
    # Pattern to retrieve all lines who are attributes [[#attr]] or properties
    valid_lines_pattern = r"^\[\[#\w+]]$|^\w+: ?.+"
    return "\n".join(re.findall(valid_lines_pattern, file, re.MULTILINE))


def parse_attr_properties_block(file: str) -> dict:
    block_pattern = r"^\[\[#\w+]]\n(?:^\w+: ?.+\n?)+"
    blocks = re.findall(block_pattern, file, re.MULTILINE)

    for block in blocks:
        block_dict = {
            "attr_name": "",
            "properties": []
        }
        attr_pattern = r"^\[\[#(\w+)]]"
        property_pattern = r"^(.+): (.+)$"

        block_dict["attr_name"] = re.search(attr_pattern, block).group(1)

        extracted_properties_buffer = re.findall(property_pattern, block, re.MULTILINE)

        for property_name, property_value in extracted_properties_buffer:
            block_dict["properties"].append(
                {"property_name": property_name,
                 "property_value": property_value
                 })
    return block_dict


def parse_compiler_config_property(line: str):
    """ Retrieves the value of a property from the 'compiler' attribute """
    if line.__contains__('cpp_compiler'):
        line = line[14:].strip()
        if line in SUPPORTED_COMPILERS:
            config.get('compiler').cpp_compiler = line
    else:
        raise UnknownProperty(line)


def parse_language_config_property(line: str):
    """ Retrieves the value of a property from the 'language' attribute """
    if line.__contains__('cpp_standard'):
        line = line[14:].strip()
        config.get('language').cpp_standard = int(line)
    elif line.__contains__('std_lib'):
        line = line[8:].strip()
        config.get('language').std_lib = line
    else:
        raise UnknownProperty(line)


def parse_build_config_property(line: str):
    """ Retrieves the value of a property from the 'build' attribute """
    if line.__contains__('output_dir'):
        line = line[12:].strip()
        config.get('build').output_dir = line
    else:
        raise UnknownProperty(line)
