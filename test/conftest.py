""" _summary_

    Configuration for the `pytest` module.
"""

import sys
import pytest
from zork.utils.constants import ZORK_CONF_AUTOG

# Discovers to pytest the project's root, which is the `zork` module
sys.path.append('./zork')


# Global fixtures to be reused accross the whole suite

@pytest.fixture(autouse=True)
def zork_conf_file() -> str:
    """ _summary_

    Fixture for get the string that represents the autogenerated
    configuration file for Zork, that is used when Zork is called
    via CLI to generate a new project.

    Returns:
        str: The autogenerated `zork.conf` file for the project by Zork itself
    """
    return ZORK_CONF_AUTOG