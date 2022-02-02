from dataclasses import dataclass


@dataclass
class CompilerConfig:
    cpp_compiler: str

@dataclass
class LanguageConfig:
    cpp_standard: int
    std_lib: str

@dataclass
class BuildConfig:
    output_dir: str
