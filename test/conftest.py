""" _summary_

    Configuration for the `pytest` module.
"""

import sys
import subprocess
import shutil
import pytest

from zork.utils.constants import ZORK_CONF_AUTOG

# Discovers to pytest the project's root, which is the `zork` module
sys.path.append('./zork')

# Shared constants for the tests
CONF_FILE_MOCK_PATH: str = './test/deps'


# Global fixtures to be reused accross the whole suite

@pytest.fixture(autouse=True)
def get_sample_zork_conf_file_as_str() -> str:
    """ _summary_

    Fixture for get the string that represents the autogenerated
    configuration file for Zork, that is used when Zork is called
    via CLI to generate a new project.

    Returns:
        str: The autogenerated `zork.conf` file for the project by Zork itself
    """
    return ZORK_CONF_AUTOG

@pytest.fixture(autouse=True)
def create_sample_zork_conf_file():
    """ _summary_

    Creates physically in the current working directory whenever the
    suite starts to run a mocked `zork.conf` file, to make it available
    to the functions on the source code that needs one to run, like,
    for example, the ones on the config_file_module
    """
    subprocess.Popen(['mkdir', CONF_FILE_MOCK_PATH]).wait()

    with open(f'{CONF_FILE_MOCK_PATH}/zork.conf', 'w', encoding='UTF-8') as zork_conf_file:
        zork_conf_file.write(ZORK_CONF_AUTOG)

    # After the tests that uses this fixture are processed, we must clean
    # the resources placed physically on the logical devices
    yield
    shutil.rmtree(CONF_FILE_MOCK_PATH)
