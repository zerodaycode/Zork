""" _summary_

    Unit tests for the code that detects if a `zork.conf` file is present
    in the cwd for start to work with `Zork++`
"""

import shutil
import subprocess
from zork.utils.workspace_scanner import find_config_file

def test_config_file_not_in_path():
    """ _summary_
        Raw test. No mocks. No `zork.conf` present on the tests dir.
        So the function should return a boolean False
    """
    assert find_config_file('.', True) is False


def test_config_file_in_project():
    """ _summary_
        We are generating an empty `zork.conf` file to test if the
        function's logic is able to process correctly the find
        of the conf file. We are NOT validating the content of the file,
        because this method just takes care about looking for the file.

        If there's no `zork.conf`, no Zork project will run ever.
    """
    PATH: str = './test/deps'
    subprocess.Popen(['mkdir', PATH]).wait()

    with open(f'{PATH}/zork.conf', 'w', encoding='UTF-8') as zork_conf_file:
        zork_conf_file.write('')
    assert find_config_file(PATH, True) is True

    # We clean up the manually mocked resources
    shutil.rmtree(PATH)
