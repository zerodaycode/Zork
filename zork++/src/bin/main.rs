use clap::Parser;
use color_eyre::{eyre::Context, Result};
use env_logger::Target;
use zork::{
    cli::{CliArgs, Command},
    config_file::ZorkConfigFile,
    utils::reader::find_config_file,
    utils::{logger::config_logger, template::create_templated_project},
};

fn main() -> Result<()> {
    color_eyre::install()?;

    // This line just remains here for debug purposes while integration tests
    // are not created
    let cli_args = CliArgs::parse_from(vec!["", "new", "example", "--git", "--compiler", "clang"]);
    // Correct one: let cli_args = CliArgs::parse();

    config_logger(cli_args.verbose, Target::Stdout).expect("Error configuring the logger");

    let config_file: String =
        find_config_file().with_context(|| "Failed to read configuration file")?;
    let _config: ZorkConfigFile = toml::from_str(config_file.as_str())
        .with_context(|| "Could not parse configuration file")?;

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
        /* Command::Build => build_project(&_config, &cli_args),
        Command::Run => {
            build_project(&_config, &cli_args);
            TODO run generated executable based on the path out property info
        } */
        Command::Test => todo!("Implement test routine"),
        Command::New {
            name,
            git,
            compiler,
        } => create_templated_project(name, git, compiler.into())
            .with_context(|| "Failed to create new project"),
    }
}
