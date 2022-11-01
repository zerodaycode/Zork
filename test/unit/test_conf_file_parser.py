""" _summary_

    Unit tests for the Zork++ configuration file parser
"""

from conftest import CONF_FILE_MOCK_PATH
from zork.config_file_parser import get_project_config
from zork.data.user_config import ProjectConfig, CompilerConfig, \
    LanguageConfig, BuildConfig, ModulesConfig, ExecutableConfig, \
    TestsConfig

def test_get_project_config():
    """ _summary_

    get_project_config is the highest level abstraction for the config file
    parser module. Is the responsible for call the different parts of the
    parsing process and then, return the results of the parsing process
    in a dictionary, where each key represents a Zork's section
    """
    expected_dict_after_parsing: dict = {
        'project': ProjectConfig(
            'calculator',
            ['Zero Day Code  # Replace this for the real authors']
        ),
        'compiler': CompilerConfig('clang++', ''),
        'language': LanguageConfig('20', 'libstdc++', True),
        'build': BuildConfig('./out'),
        'modules': ModulesConfig(
            '<project_name>/ifc/',
            ['*.cppm'],
            '<project_name>/src/',
            ['math.cpp', 'math2.cpp=[math]']
        ),
        'executable': ExecutableConfig('calculator', '', ['*.cpp'], 'true'),
        'tests': TestsConfig('zork_proj_tests', '', ['*.cpp'], 'true')
    }

    parsed_config = get_project_config(
        CONF_FILE_MOCK_PATH, False
    )

    assert expected_dict_after_parsing['project'].name == \
        parsed_config['project'].name
    assert expected_dict_after_parsing['project'].authors[0] == \
        parsed_config['project'].authors[0]

    assert expected_dict_after_parsing['compiler'].cpp_compiler == \
        parsed_config['compiler'].cpp_compiler
    assert expected_dict_after_parsing['compiler'].system_headers_path == \
        parsed_config['compiler'].system_headers_path

    assert expected_dict_after_parsing['language'].cpp_standard == \
        parsed_config['language'].cpp_standard
    assert expected_dict_after_parsing['language'].std_lib == \
        parsed_config['language'].std_lib
    assert expected_dict_after_parsing['language'].modules == \
        parsed_config['language'].modules

    assert expected_dict_after_parsing['build'].output_dir == \
        parsed_config['build'].output_dir

    assert expected_dict_after_parsing['modules'].base_ifcs_dir == \
        parsed_config['modules'].base_ifcs_dir
    assert expected_dict_after_parsing['modules'].interfaces == \
        parsed_config['modules'].interfaces
    assert expected_dict_after_parsing['modules'].base_impls_dir == \
        parsed_config['modules'].base_impls_dir
    assert expected_dict_after_parsing['modules'].implementations == \
        parsed_config['modules'].implementations

    assert expected_dict_after_parsing['executable'].executable_name == \
        parsed_config['executable'].executable_name
    assert expected_dict_after_parsing['executable'].sources_base_path == \
        parsed_config['executable'].sources_base_path
    assert expected_dict_after_parsing['executable'].sources == \
        parsed_config['executable'].sources
    assert expected_dict_after_parsing['executable'].auto_execute == \
        parsed_config['executable'].auto_execute

    assert expected_dict_after_parsing['tests'].tests_executable_name == \
        parsed_config['tests'].tests_executable_name
    assert expected_dict_after_parsing['tests'].sources_base_path == \
        parsed_config['tests'].sources_base_path
    assert expected_dict_after_parsing['tests'].sources == \
        parsed_config['tests'].sources
    assert expected_dict_after_parsing['tests'].auto_run_tests == \
        parsed_config['tests'].auto_run_tests
