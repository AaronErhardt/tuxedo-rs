use clap::{Parser, Subcommand};

/// CLI to interact with tailord
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Opts {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Command {
    /// Profile commands
    Profile {
        #[command(subcommand)]
        profile_cmd: ProfileCommand,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ProfileCommand {
    /// List profile names
    List,

    /// Set the active profile
    Set {
        /// The name of the profile to set (see: list)
        #[arg()]
        name: String,
    },
}
