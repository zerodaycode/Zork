""" _summary_

    Unit tests for the code that detects if a `zork.conf` file is present
    in the cwd for start to work with `Zork++`
"""

import shutil
import subprocess

from conftest import CONF_FILE_MOCK_PATH

from zork.utils.workspace_scanner import find_config_file

def test_config_file_not_in_path():
    """ _summary_
        Raw test. No mocks. No `zork.conf` present on the tests dir.
        So the function should return a boolean False
    """
    assert find_config_file('.', False) is False


def test_config_file_in_project():
    """ _summary_

        Tests if the function's logic is able to process correctly the find
        of the conf file. We are NOT validating the content of the file,
        because this method just takes care about looking for the file.
    """
    assert find_config_file(CONF_FILE_MOCK_PATH, False) is True
    assert find_config_file(CONF_FILE_MOCK_PATH, True) is True
