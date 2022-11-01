""" _summary_

    Unit tests for the code that detects if a `zork.conf` file is present
    in the cwd for start to work with `Zork++`
"""

from zork.utils.workspace_scanner import find_config_file

def test_config_file_retriever():
    """ _summary_
        Raw test. No mocks. No `zork.conf` present on the tests dir.
        So the function should return a boolean False
    """
    assert find_config_file('.', True) is False
