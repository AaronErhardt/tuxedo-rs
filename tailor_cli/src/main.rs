mod cli;
mod profile;

use clap::Parser;
use eyre::Result;

use crate::cli::{Command, Opts};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Opts::parse();
    if let Some(Command::Profile { profile_cmd }) = args.command {
        profile::handle(profile_cmd).await?;
    }
    Ok(())
}
