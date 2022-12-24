pub(crate) use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub(crate) struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Update existing translation files
    Update,
    /// Generate translation files named with language tag
    Generate {
        locale: Vec<String>,
    }
}
