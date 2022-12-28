pub mod command;

use clap::Parser;
use command::Command;


/// TODO
/// 
/// ```rust
/// use zork::config_cli::CliArgs;
/// use clap::Parser;
/// let parser = CliArgs::parse_from(["", "-v", "-t"]);
/// assert_eq!(1, parser.verbose);
/// assert_eq!(true, parser.tests);
/// ```
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[arg(short, long)]
    pub tests: bool,
    #[command(subcommand)]
    pub command: Option<Command>
}

