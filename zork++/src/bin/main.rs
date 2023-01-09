use clap::Parser;
use env_logger::Target;
use zork::{
    cli::{CliArgs, Command},
    utils::{logger::config_logger, template::create_templated_project},
};

fn main() {
    let cli_args = CliArgs::parse();
    config_logger(cli_args.verbose, Target::Stdout).expect("Error configuring the logger");

    match cli_args.command {
        Command::Test => todo!("Implement test routine"),
        Command::New {
            name,
            git,
            compiler,
        } => create_templated_project(name, git, compiler.into()),
    }
}
