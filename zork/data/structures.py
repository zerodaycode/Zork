from dataclasses import dataclass
from typing import Any

"""[summary]
    Provides classes to store the options selected by the 
    user in the configuration file
"""

@dataclass
class CompilerConfig:
    cpp_compiler: str

    def set_property(self, property_name: str, value: Any):
        if property_name == 'cpp_compiler':
            self.cpp_compiler = value

@dataclass
class LanguageConfig:
    cpp_standard: int
    std_lib: str

    def set_property(self, property_name: str, value: Any):
        if property_name == 'cpp_standard':
            self.cpp_standard = value
        elif property_name == 'std_lib':
            self.std_lib = value

@dataclass
class BuildConfig:
    output_dir: str

    def set_property(self, property_name: str, value: Any):
        if property_name == 'output_dir':
            self.output_dir = value