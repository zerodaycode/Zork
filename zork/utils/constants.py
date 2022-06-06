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


# Content for the autogenerated files

ZORK_CONF_AUTOG: str = \
    """# This file it's autogenerated as an example of a Zork config file

[[#project]]
name: <project_name>
authors: Zero Day Code  # Replace this for the real authors

[[#compiler]]
cpp_compiler: clang++

[[#language]]
cpp_standard: 20
std_lib: libc++
modules: true

[[#build]]
output_dir: ./out

[[#executable]]
executable_name: <autogenerated_executable>
sources: *.cpp
auto_execute: true

[[#modules]]
base_ifcs_dir: <project_name>/ifc/
interfaces: *.cppm
base_impls_dir: <project_name>/src/
implementations: math.cpp;math2.cpp=[math]
    """

MAIN_CPP: str = \
    """import <iostream>;
import math;

int main() {
    std::cout << "Hello from an autogenerated Zork project!" << std::endl;

    std::cout << "RESULT '+': " << math::sum(2, 8) << std::endl;
    std::cout << "RESULT '-': " << math::substract(8, 2) << std::endl;
    std::cout << "RESULT '*': " << math::multiply(2, 8) << std::endl;
    std::cout << "RESULT '/': " << math::divide(2, 2) << std::endl;

    return 0;
}
    """

INTERFACE_MOD_FILE: str = \
    """export module math;

export namespace math {
    int sum(int num1, int num2);

    int multiply(int num1, int num2);

    int substract(int num1, int num2);

    int divide(int num1, int num2);
}
    """

SRC_MOD_FILE: str = \
    """module math;


// Implementation of the definitions on the module unit interface
// for the sum and multiply math operations

namespace math {
    int sum(int num1, int num2) {
        return num1 + num2;
    }

    int multiply(int num1, int num2) {
        return num1 * num2;
    }
}
    """

SRC_MOD_FILE_2: str = \
    """module math;


// Implementation of the definitions on the module unit interface
// for the substract and divide math operations

namespace math {
    int substract(int num1, int num2) {
        return num1 - num2;
    }

    int divide(int num1, int num2) {
        return num1 / num2;
    }
}
    """
