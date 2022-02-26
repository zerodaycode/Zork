""" _summary_

    Provides the command line interface for be consumed as a client
"""

import argparse


def command_line_interface():
    """ Manages to take the available Zork program options as
        command line arguments """

    parser = argparse.ArgumentParser(description='The Zork CLI')

    parser.add_argument(
        '-v',
        '--verbose',
        dest='verbose',
        action='store_true',
        help='Controls the information sent to stdout/stderr'
    )

    return parser.parse_args()
