from utils.exceptions import DuplicatedAttribute, MissedMandatoryAttributes, \
    UnknownAttribute, UnknownProperty, ErrorFileFormat
from data.structures import CompilerConfig, LanguageConfig, BuildConfig

from utils.constants import *
from utils.regex_patterns import RE_ATTRIBUTES, RE_VALID_LINE_FORMAT

import re

<<<<<<< HEAD
"""
    [summary] This file provides the necessary mechanisms to parse the Zork++
        configuration file.
        It's made of just basic functions that parses the two kind of tokens
        that conforms the logic of the application data, which are:
            - Attributes -> [[#Attribute]]
            - Property -> <property_name>: <value>
"""

# Initializes the map with the config values and provide default values
config: dict = {
    'compiler' : CompilerConfig('clang'),
    'language' : LanguageConfig(20, 'libstdc++'),
    'build' : BuildConfig('./build')
}

def get_project_config(root_path: str) -> dict:
    """ Parses the file looking for a kind of AST token tokens """
=======

def get_project_config(root_path: str) -> dict:
    """Parses the file looking for a kind of AST token tokens"""

>>>>>>> dev-regex-parser
    # Open the configuration file in 'read-only' mode
    config_file = read_config_file_lines(root_path)

    # Check if the config file format it's valid
    check_valid_config_file(config_file)

<<<<<<< HEAD
def read_config_file_lines(root_path: str):
    """ Reads line by line the configuration file, distinguishing between
        attributes and properties """
    with open(root_path + '/' + CONFIGURATION_FILE_NAME, 'r') as config_file:
        # Get all the lines written in the conf file
        lines = config_file.readlines()
        # Tracks what attribute (or it's properties) are being parsed
        # when an attribute is discovered
        current_attr: str = ''
        for line in lines:
            line = line.rstrip('\n')
            if line.startswith('[[#'):
                # If starts with the '[[' symbols,
                # it's a line with a section attribute identifier
                find_section_attribute(line)
                current_attr = line
            elif line == '' or line.startswith("#"):
                pass
            else:
                # Then, it should be a property
                property_parser(line, current_attr)

        # ['[[#\nmypropiedad: a', '[[#...]]] 

def check_mandatory_attributes():
    """ Checks if all the defined as 'mandatory attribute' elements
        are present in the configuration file
=======
    # If the config file it's OK, the we can retrieve all the config sections
    return get_sections(config_file)



def read_config_file_lines(root_path: str) -> list:
    """ Get all the lines written in the conf file """
    with open(root_path + '/' + CONFIGURATION_FILE_NAME, 'r') as config_file:
        return config_file.readlines()


def check_valid_config_file(config_file: list):
    """ # TODO """
    # Parses the file to check if it's valid
    for idx, line in enumerate(config_file):
        line = line.strip()

        if line and not re.match(RE_VALID_LINE_FORMAT, line):
            raise ErrorFileFormat(idx + 1, line)


def clean_file(file: str) -> list:
    """Clean the file and retrieve only lines with attribute or property format"""
    # Pattern to retrieve all lines who are attributes [[#attr]] or properties
    valid_lines_pattern = r"^\[\[#\w+]]$|^\w+: ?.+"
    return re.findall(
            valid_lines_pattern, file, re.MULTILINE
        )


def parse_attr_properties_block(file: str) -> dict:
    block_pattern = r"^\[\[#\w+]]\n(?:^\w+: ?.+\n?)+"
    blocks = re.findall(block_pattern, file, re.MULTILINE)

    retrieved_data = {}

    for block in blocks:
        attr_pattern = r"^\[\[#(\w+)]]"
        property_pattern = r"^(.+): (.+)$"

        attribute_identifier = re.search(attr_pattern, block).group(0)
        extracted_properties = re.findall(property_pattern, block, re.MULTILINE)

        properties: list = []
        for property_name, property_value in extracted_properties:
            properties.append(
                {"property_name": property_name,
                 "property_value": property_value
                }
            )

        retrieved_data[attribute_identifier] = properties

    return retrieved_data


def get_sections(config_file: str) -> dict:
    """ Recovers the sections described in the config file, returning a dict with 
        the instances of the dataclasses designed for carry the final data 
        to the compiler """

    # Initializes the map with the config values and provide default values
    config: dict = {
        'compiler' : CompilerConfig('clang'),
        'language' : LanguageConfig(20, 'libstdc++'),
        'build' : BuildConfig('./build')
    }

    cleaned_config_file: list = clean_file("".join(config_file))
    attr_ppt_collection = parse_attr_properties_block('\n'.join(cleaned_config_file))

    """ 
        Once we have parsed and cleaned the sections founded on the config file, we can 
        start match them against the valid ones (the ones allowed by Zork).
        Until here, we only validated that the code written on the conf file it's
        syntanctically correct acording the rules provided by the program. 
        Now we must discover if the retrived data it's also available and exists
        inside Zork.
>>>>>>> dev-regex-parser
    """

    # For every attribute and property founded in the config file, now stored as a 
    # dict with the attribute name as a key and the properties as an inner dict
    for attribute, ppt_list in attr_ppt_collection.items():
        for section in PROGRAM_SECTIONS: # For every section available on the program
            if section.identifier == attribute:
                # Then we found a valid whole section to serialize into the dataclasses.
                # Remove the '[[#' and ']' from the name, to match it against the dict
                # that holds the instances of the configuration classes
                print('\nSection: ' +  attribute)
                attr_identifier = attribute[3:-2]
                # Now matches the available properties of the current attribute on the loop
                # against the retrieved ones in the current 'attribute' instace
                for property in ppt_list: # For every property discovered on the conf file
                    # For every property available in the program for the current section
                    for designed_ppt in section.properties: # Properties instance
                        designed_ppt_identifier = designed_ppt.as_dict()['identifier']

<<<<<<< HEAD
def find_section_attribute(line: str):
    """ Discovers written attributes and reports the mandatory missing ones """
    
    # Check for duplicates
    if line in attributes_found:
        raise DuplicatedAttribute(line)

    if line == COMPILER_ATTR: # Mandatory
        attributes_found.append(COMPILER_ATTR)
        mandatory_attributes_found.append(COMPILER_ATTR)
    elif line == LANGUAGE_ATTR: # Mandatory
        attributes_found.append(LANGUAGE_ATTR)
        mandatory_attributes_found.append(LANGUAGE_ATTR)
    elif line == BUILD_ATTR: # Optional, it has a default
        attributes_found.append(BUILD_ATTR)
    else:
        raise UnknownAttribute(line)

def property_parser(line: str, current_attribute: str) -> None:
    """ Parses a given line from some buffer input of reading the config file
        trying to retrieve a valid value to some property of some attribute"""
    if current_attribute == COMPILER_ATTR:
        parse_compiler_config_property(line)
    elif current_attribute == LANGUAGE_ATTR:
        parse_language_config_property(line)
    elif current_attribute == BUILD_ATTR:
        parse_build_config_property(line)

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
        
=======
                        if designed_ppt_identifier == property["property_name"]:
                            print(f'PROPERTIES in config file: {property}')
                            print(f'MATCH. Value: {property["property_value"]}')
                            # Kind of templating metaprogramming, taking advantage of
                            # the Python's duck typing system, due to the lack of real 
                            # generic programming tools
                            config[attr_identifier].set_property(property["property_name"], property["property_value"])
    print('\n')
    return config
>>>>>>> dev-regex-parser
