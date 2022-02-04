from exceptions import DuplicatedAttribute, MissedMandatoryAttributes, \
    UnknownAttribute, UnknownProperty
from structures import CompilerConfig, LanguageConfig, BuildConfig
from constants import *

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
    # Open the configuration file in 'read-only' mode
    read_config_file_lines(root_path)
    # Check mandatory tokens found
    check_mandatory_attributes()

    return config

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
    """
    # Check if they are present. If not, append it to a tracknig list
    for attr in MANDATORY_ATTRIBUTES:
        if attr not in mandatory_attributes_found:
            missed_mandatory_attributes.append(attr)
    # Raise an exception containing all the missed mandatory attributes
    if len(missed_mandatory_attributes) != 0:
        raise MissedMandatoryAttributes(missed_mandatory_attributes)


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
        