import os
from utils.constants import CONFIGURATION_FILE_NAME


def find_config_file(root_path: str, verbose) -> bool:
    """
        Tries to find in the directory where this script it's placed
        a `zork.conf` file that identifies the "config" file
        with the pseudo-language in there
    """
    for file in os.listdir(root_path):
        if file == CONFIGURATION_FILE_NAME:
            if verbose:
                print(
                    '[SUCCESS]: Configuration file founded on: ' + os.getcwd()
                )
            return True
    return False
