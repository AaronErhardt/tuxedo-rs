use colored::Colorize;
use eyre::Result;
use tailor_client::TailorConnection;

use crate::cli::ProfileCommand;

/// Handle profile commands
pub(crate) async fn handle(cmd: ProfileCommand) -> Result<()> {
    let connection = TailorConnection::new().await?;
    match cmd {
        ProfileCommand::List => {
            let active_profile = connection.get_active_global_profile_name().await?;
            let inactive_profiles: Vec<String> = connection
                .list_global_profiles()
                .await?
                .into_iter()
                .filter(|name| name != &active_profile)
                .collect();
            let active_profile_str = format!("{} (active)", active_profile).bold().green();
            println!("{}\n{}", active_profile_str, inactive_profiles.join("\n"));
        }
        ProfileCommand::Set { name } => connection.set_active_global_profile_name(&name).await?,
    }
    Ok(())
}
