pub(crate) use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, bin_name = "cargo")]
pub(crate) struct Args {
    #[command(subcommand)]
    command: SelfCommand,
}

#[derive(Parser)]
pub(crate) struct InnerArgs {
    #[command(subcommand)]
    pub command: Command,

    /// Root directory of your project
    #[arg(short, long, default_value_t = { "./".to_string() })]
    pub root: String,
}

#[derive(Subcommand)]
enum SelfCommand {
    R18(InnerArgs)
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Update existing translation files
    Update {
        /// Remove unused translations
        #[arg(short, long, default_value_t = false)]
        rm_unused: bool,
    },
    /// Generate translation files named with language tag
    Generate {
        locales: Vec<String>,
    }
}

impl Args {
    pub fn inner_args(self) -> InnerArgs {
        let SelfCommand::R18(args) = self.command;
        args
    }
}
