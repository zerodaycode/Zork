"""[summary]

    Contains several functions with pre-build code schemas for interfacing
    a logger system that acts as an informative process of the project
"""

import subprocess
import time
from .constants import OS, OS_release, OS_architecture, OS_arch_linkage


def initial_log():
    """ Greets the users when the build task is started, an log
        some useful info about the OS where the program it's running """
    print(
        "\n[INFO]: Starting a new C++ compilation job with Zork on " +
        f"[{OS} {OS_release}, {OS_architecture}, {OS_arch_linkage}]"
    )


def log_process_result(start_time: int, process_result):
    """ Logs the result and the total time spent on run the process and """
    process_time = (time.time_ns() // 1_000_000) - start_time
    process_duration_log = 'Total time spent on the process: ' + \
        f'{process_time} ms'

    if (process_result == 0):
        success_log = '[SUCCESS]: Compilation job finished. ' + \
            process_duration_log
        print(success_log + '\n')
        notify_process_result_on_system_popup(success_log)
    else:
        fail_log = '\n[ERROR]: Compilation job FAILED. ' + process_duration_log
        print(fail_log)
        notify_process_result_on_system_popup(fail_log + '\n')


def notify_process_result_on_system_popup(message: str):
    """ Shows a system popup in the desktop with the result of the process """
    if OS == 'Linux':
        subprocess.Popen(['notify-send', message])
    else:
        # TODO Use ToastNotifier for Windows
        pass


def show_final_config_values(config: dict):
    """ Show the arguments that will be passed to the compiler """
    print(f'\nCompiler: {config.get("compiler")}')
    print(f'Language: {config.get("language")}')
    print(f'Build: {config.get("build")}')
    print(f'Executable: {config.get("executable")}\n')
