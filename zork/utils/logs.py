"""[summary]

    Contains several functions with pre-build code schemas for interfacing
    a logger system that acts as an informative process of the project
"""
import time
from .constants import OS, OS_release, OS_architecture, OS_arch_linkage

def initial_log():
    """ Greets the users when the build task is started, an log
        some useful info about the OS where the program it's running """
    print(
        f"\n[INFO]: Starting a new C++ compilation job with Zork on " + \
           f"[{OS} {OS_release}, {OS_architecture}, {OS_arch_linkage}]"
    )

def log_process_result(start_time: int, process_result):
    process_time = (time.time_ns() // 1_000_000) - start_time
    """ Logs the result and the total time spent on run the process and """
    process_duration_log = f"Total time spent on the process: {process_time} ms"
    if (process_result == 0):
        print('[SUCCESS]: Compilation job finished. ' + process_duration_log)
    else:
        print('\n[ERROR]: Compilation job FAILED. ' + process_duration_log)

def show_final_config_values(config: dict):
    """ Show the arguments that will be passed to the compiler """
    print(f'\nCompiler: {config.get("compiler")}')
    print(f'Language: {config.get("language")}')
    print(f'Build: {config.get("build")}')
    print(f'Executable: {config.get("executable")}\n')