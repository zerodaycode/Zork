"""[summary]

    Contains several functions with pre-build code schemas for interfacing
    a logger system that acts as an informative process of the project
"""
from constants import OS, OS_release, OS_architecture, OS_arch_linkage

def initial_log():
    """ Greets the users when the build task is started, an log
        some useful info about the OS where the program it's running """
    print(
        f"\n[INFO]: Starting a new C++ compilation job with Zork on " + \
           f"[{OS} {OS_release}, {OS_architecture}, {OS_arch_linkage}]"
    )