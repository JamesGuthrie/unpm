use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "unpm", about = "Lightweight vendoring of static assets")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Add a dependency (interactive)
    Add {
        /// Package specifier: npm name (e.g. htmx.org) or gh:user/repo
        package: String,
        /// Package version (default: latest)
        #[arg(long)]
        version: Option<String>,
        /// File path within the package
        #[arg(long)]
        file: Option<String>,
    },
    /// Fetch all dependencies
    Install,
    /// Verify vendored files and check for CVEs
    Check {
        /// Allow known vulnerabilities
        #[arg(long)]
        allow_vulnerable: bool,
    },
    /// List all dependencies
    List,
    /// Show dependencies with newer versions available
    Outdated,
    /// Remove a dependency
    Remove {
        /// Package name to remove
        package: String,
    },
}
