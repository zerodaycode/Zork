use clap::Parser;
use env_logger::Target;
use zork::{
    cli::{CliArgs, Command},
    utils::{logger::config_logger, template::create_templated_project},
};

fn main() {
    let cli_args = CliArgs::parse_from(vec!["", "-vv", "new", "--git"]);
    config_logger(cli_args.verbose, Target::Stdout).expect("Error configuring the logger");

    match cli_args.command {
        Some(Command::Test) => todo!("Implement test routine"),
        Some(Command::New { name, git, compiler }) => create_templated_project(name, git, compiler.into()),
        None => todo!("Show usage"),
    }
}
