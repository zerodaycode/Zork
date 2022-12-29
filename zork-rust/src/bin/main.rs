use zork::config_cli::CliArgs;
use clap::Parser;


fn main() {
    println!("Hello, world!");
    let argument_help = vec!["","--help"];
    CliArgs::parse_from(argument_help);


    let arguments = vec!["","tests"];
    let parser = CliArgs::parse_from(arguments);
    println!("{:?}",parser);


    let arguments = vec!["","-vv"];
    let parser = CliArgs::parse_from(arguments);
    println!("{:?}",parser);

    let arguments = vec![
        "","-n","ZeroDayCodeTemplate","-l","-g","-c","clang"
    ];
    let parser = CliArgs::parse_from(arguments);
    println!("{:?}",parser);

}
