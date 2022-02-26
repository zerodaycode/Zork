import platform


""" Constant definitions across the whole program """
PROJECT_VERSION: str = '0.3.0'
CONFIGURATION_FILE_NAME: str = 'zork.conf'

""" Retrieves the OS available info """
OS = platform.system()
OS_release = platform.release()
OS_version = platform.version()  # Unused for the moment
OS_architecture = platform.architecture()[0]
OS_arch_linkage = platform.architecture()[1]
