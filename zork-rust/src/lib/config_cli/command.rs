///! TODO module level doc
use clap::Subcommand;

/// [`Command`] -  The core enum commands
#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Command {
    /// Run tests
    Tests,
}
