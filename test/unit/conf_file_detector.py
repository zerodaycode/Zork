""" _summary_

    Unit tests for the code that detects if a `zork.conf` file is present
    in the cwd for start to work with `Zork++`
"""

from zork.utils.workspace_scanner import find_config_file

def config_file_retriever_test():
    """ _summary_
        Testing the behaviour of the function that searches the `zork.conf` file
        in the cwd for start to work with `Zork++`
    """
    assert find_config_file('.', True) is True
