from dataclasses import dataclass


@dataclass
class CompilerConfig:
    """ Holds the data relative to the compiler """
    cpp_compiler: str

@dataclass
class LanguageConfig:
    """ Holds the data relative to the language specs """
    cpp_standard: int
    std_lib: str

@dataclass
class BuildConfig:
    """ Manages the parameters of the compiler output files """
    output_dir: str
