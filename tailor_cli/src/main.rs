mod cli;
mod profile;

use clap::Parser;
use eyre::Result;

use crate::cli::{Opts, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Opts::parse();
    match args.command {
        Some(Command::Profile { profile_cmd }) => {
            profile::handle(profile_cmd).await?
        }
        None => {}
    }
    Ok(())
}
