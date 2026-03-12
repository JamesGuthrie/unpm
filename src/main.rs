use clap::Parser;
use unpm::cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.debug {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .format_target(false)
            .init();
    }

    match cli.command {
        Command::Add {
            package,
            version,
            file,
        } => {
            unpm::commands::add(&package, version.as_deref(), file.as_deref()).await?;
        }
        Command::Install => {
            unpm::commands::install().await?;
        }
        Command::Check {
            allow_vulnerable,
            fail_on_outdated,
        } => {
            unpm::commands::check(allow_vulnerable, fail_on_outdated).await?;
        }
        Command::List => {
            unpm::commands::list()?;
        }
        Command::Outdated => {
            unpm::commands::outdated().await?;
        }
        Command::Update {
            package,
            version,
            latest,
        } => {
            unpm::commands::update(package.as_deref(), version.as_deref(), latest).await?;
        }
        Command::Remove { package } => {
            unpm::commands::remove(&package)?;
        }
    }

    Ok(())
}
