use clap::Parser;
use color_eyre::{eyre::Context, Result};
use env_logger::Target;
use zork::{
    cli::{CliArgs, Command},
    compiler::build_project,
    utils::{logger::config_logger, template::create_templated_project},
};

fn main() -> Result<()> {
    color_eyre::install()?;

    // This line just remains here for debug purposes while integration tests
    // are not created
    let cli_args = CliArgs::parse_from(vec![
        "",
        "-vv",
        "new",
        "example",
        "--git",
        "--compiler",
        "clang",
    ]);
    // let cli_args = CliArgs::parse();

    config_logger(cli_args.verbose, Target::Stdout).expect("Error configuring the logger");

    /* TODO We should build the project normally (taking in consideration the implementation
    of a cache based on the metadata of the source code files), and then probably
    matching this available options, choose the final alternative

    For example, we may match the behaviour of cargo, using Zork++ like:
    ~ zork++ test => this should generate the executable for the test suite (autoexecution
        must probably be assumed)
    ~ zork++ build => this should only build the project and generate the executable (if applies)
    ~ zork++ run => zork++ build + run the generated binary
    */
    match cli_args.command {
        // TODO provisional Ok wrapper, pending to implement color eyre err handling
        Command::Build => build_project(&cli_args).with_context(|| "Failed to build project"),
        /*Command::Run => {
            build_project(&_config, &cli_args);
            TODO run generated executable based on the path out property info
        } */
        Command::Test => todo!("Implement test routine"),
        Command::New {
            name,
            git,
            compiler,
        } => create_templated_project(&name, git, compiler.into())
            .with_context(|| "Failed to create new project"),
    }
}
