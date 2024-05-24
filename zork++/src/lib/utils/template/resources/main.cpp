const char* compiler =
#if defined(__clang__)
    "Clang";
    import std;
#elif (defined(__GNUC__) || defined(__GNUG__)) && !defined(__clang__)
    "GCC";
    import <iostream>;
#elif defined(_MSC_VER)
    "MSVC";
    import std;
#endif

import math;
import partitions;

int main() {
    std::cout << "\nHello from an autogenerated Zork project!" << std::endl;
    std::cout << "The program is running with: " << compiler << std::endl << std::endl;
    std::cout << "RESULT of 2 + 8 = " << math::sum(2, 8) << std::endl;
    std::cout << "RESULT of 8 - 2 = " << math::subtract(8, 2) << std::endl;
    std::cout << "RESULT of 2 * 8 = " << math::multiply(2, 8) << std::endl;
    std::cout << "RESULT of 2 / 2 = " << math::divide(2, 2) << std::endl << std::endl;

    std::cout << "Testing interface module partitions, by just returning the number: "
        << just_a_42() << std::endl;

    return 0;
}

