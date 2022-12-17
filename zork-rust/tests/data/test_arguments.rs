use zork::data::arguments::{FileAttributes, ProjectAttribute, CompilerAttibute, SupportCompiler, ModulesAttribute, TestAttribute, ExecutableAttribute, LanguageAttribute, BuildAttribute};



#[test]
fn parse_arguments_toml_project_attribute(){
    let zork_config: Result<ProjectAttribute, toml::de::Error> = toml::from_str(
        r#"
            name = 'calculator'
            authors = [
                'Zero Day Code  # Replace this for the real authors'
            ]"#
    );

    match &zork_config {
        Ok(params) => {
            assert_eq!(params.name.as_str(),"calculator");
            assert!(params.authors.is_some());

            assert_eq!( params.authors.clone().unwrap().len(), 1 );

            assert_eq!( params.authors.clone().unwrap().get(0).unwrap(),
                "Zero Day Code  # Replace this for the real authors",
                "Author not equals"
            );
        },
        Err(type_error) => {
            assert!( false, "Error parse &str: {:?} and error is : {:?}",zork_config, type_error )
        },
    }


    let zork_config: Result<ProjectAttribute, toml::de::Error> = toml::from_str(
        r#"
            "#
    );

    match &zork_config {
        Ok(_) => {
            assert!( false, "Can parse toml to object" )
        },
        Err(_) => {
            assert!(true)
        },
    }
}

#[test]
fn parse_arguments_compiler_attribute(){
    let compiler_attribute:Result<CompilerAttibute, toml::de::Error> = toml::from_str(r#"
        cpp_compiler = 'clang++'
        extra_args = 'params'
        system_headers_path = 'system_headers_path'
    "#); 


    match &compiler_attribute {
        Ok(params) => {
            assert_eq!(params.cpp_compiler,SupportCompiler::CLANG);
            assert_eq!(params.extra_args.as_ref().unwrap().as_str(),"params");
            assert_eq!(params.system_headers_path.as_ref().unwrap().as_str(),"system_headers_path")
        },
        Err(type_error) => {
            assert!( false, "Error parse &str: {:?} and error is : {:?}",compiler_attribute, type_error )
        },
    }


    let compiler_attribute:Result<CompilerAttibute, toml::de::Error> = toml::from_str(r#"
        cpp_compiler = 'clang++'
    "#); 

    match &compiler_attribute {
        Ok(params) => {
            assert_eq!( params.cpp_compiler, SupportCompiler::CLANG, "Cant parse cpp_compiler" );
            assert_eq!( params.extra_args, None, "Found and parse extra_args" );
            assert_eq!( params.system_headers_path, None, "Found and parse system_headers_path" )
        },
        Err(type_error) => {
            assert!( false, "Error parse &str: {:?} and error is : {:?}",compiler_attribute, type_error )
        },
    }

}

#[test]
fn parse_arguments_toml_all_arguments_attribute(){
    let zork_config : Result< FileAttributes, toml::de::Error > =  toml::from_str( get_string_parse_config() );

    match &zork_config {
        Ok( params ) => {
            let project = ProjectAttribute {
                name: "calculator".to_string(),
                authors: Some( 
                    vec![
                        "Zero Day Code  # Replace this for the real authors".to_string()
                    ]
                )
            };
            assert_eq!(params.project, project );

            let compiler = CompilerAttibute {
                cpp_compiler: SupportCompiler::CLANG,
                extra_args: Some("aaa".to_string()),
                system_headers_path: Some("system_headers_path".to_string()),
            };
            assert_eq!(params.compiler, compiler);

            let language = LanguageAttribute {
                cpp_standard: 20 ,
                std_lib: Some("libc++".to_string()),
                modules: Some(true),
            };
            assert_eq!(params.language, language);

            let build = BuildAttribute {
                output_dir: Some("./out".to_string()),
            };
            assert_eq!(params.build, Some(build));

            assert!( params.executable.is_some() );

            let executable = ExecutableAttribute{
                executable_name: Some("calculator".to_string()),
                sources_base_path:  Some("asd".to_string()),
                sources: Some(
                    vec![ "*.cpp".to_string()]
                ),
                auto_execute: Some(true),
                extra_args: Some("aaa".to_string()) ,
            };
            assert_eq!( params.executable, Some(executable) );

            assert!( params.modules.as_ref().is_some() );
            let modules = ModulesAttribute {
                base_ifcs_dir:  Some("calculator/ifc/".to_string()) ,
                interfaces: Some( vec![ "*.cppm".to_string() ] ) ,
                base_impls_dir: Some("calculator/src/".to_string()),
                implementations: Some(
                    vec![ "math.cpp".to_string(), "math2.cpp=[math]".to_string() ]
                ),
            };
            assert_eq!(params.modules.as_ref(), Some(&modules));


            assert!( params.tests.is_some() );
            let test = TestAttribute {
                tests_executable_name: Some(
                    vec!["asd".to_string()]
                ),
                sources_base_path: Some(
                    vec!["asd".to_string()]
                ),
                sources: Some(
                    vec!["asd".to_string()]
                ),
                auto_run_tests: Some(true),
                extra_args: Some(
                    "asd".to_string()
                ),
            };
            assert_eq!( params.tests, Some(test));

        },
        Err( type_error ) => {
            assert!(false, "Error parse &str: {:?} and error is : {:?}", zork_config, type_error)
        },
    }
}




fn get_string_parse_config<'a> () -> &'a str {
    return r#"
    [project]
    name = 'calculator'
    authors =  [
        'Zero Day Code  # Replace this for the real authors'
        ]

    [compiler]
    cpp_compiler =  'clang++'
    extra_args = 'aaa'
    system_headers_path = 'system_headers_path'

    [language]
    cpp_standard = 20
    std_lib = 'libc++'
    modules =  true

    [build]
    output_dir = './out'

    [executable]
    executable_name = 'calculator'
    sources = [
        '*.cpp'
    ]
    auto_execute = true
    extra_args = 'aaa'
    sources_base_path = 'asd'

    [modules]
    base_ifcs_dir = 'calculator/ifc/'
    interfaces = [
        '*.cppm'
    ]
    base_impls_dir = 'calculator/src/'
    implementations = [
        'math.cpp',
        'math2.cpp=[math]'
    ]

    [tests]
    tests_executable_name = [
        "asd"
    ]
    sources_base_path = [
        "asd"
    ]
    sources = [
        "asd"
    ]
    auto_run_tests = true
    extra_args = 'asd'
    "#;
}