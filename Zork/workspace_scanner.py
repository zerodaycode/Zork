import os
from constants import CONFIGURATION_FILE_NAME

def find_config_file() -> bool:
    """
        Tries to find in the directory where this script it's placed
        a *.txt file that identifies the "config" file with the pseudo-language in there
    """
    print(f'CWD: {os.getcwd()}')
    for file in os.listdir('./Zork'):
        print(f'File: {file}')
        if file == CONFIGURATION_FILE_NAME:
            return True
    return False
