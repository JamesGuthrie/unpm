use clap::Parser;
use unpm::cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Add { package, version, file } => {
            unpm::commands::add(&package, version.as_deref(), file.as_deref()).await?;
        }
        Command::Install => {
            unpm::commands::install().await?;
        }
        Command::Check { allow_vulnerable, fail_on_outdated } => {
            unpm::commands::check(allow_vulnerable, fail_on_outdated).await?;
        }
        Command::List => {
            unpm::commands::list()?;
        }
        Command::Outdated => {
            unpm::commands::outdated().await?;
        }
        Command::Update { package, version } => {
            unpm::commands::update(&package, version.as_deref()).await?;
        }
        Command::Remove { package } => {
            unpm::commands::remove(&package)?;
        }
    }

    Ok(())
}
