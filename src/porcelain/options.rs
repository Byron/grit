use std::{ffi::OsString, path::PathBuf};

use clap::{AppSettings, Clap};

#[derive(Debug, Clap)]
#[clap(about = "The rusty git", version = clap::crate_version!())]
#[clap(setting = AppSettings::SubcommandRequired)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Args {
    /// Do not display verbose messages and progress information
    #[clap(long, short = 'q')]
    pub quiet: bool,

    /// Bring up a terminal user interface displaying progress visually
    #[clap(long, conflicts_with("quiet"))]
    pub progress: bool,

    /// The progress TUI will stay up even though the work is already completed.
    ///
    /// Use this to be able to read progress messages or additional information visible in the TUI log pane.
    #[clap(long, conflicts_with("quiet"), requires("progress"))]
    pub progress_keep_open: bool,

    #[clap(subcommand)]
    pub cmd: Subcommands,
}

#[derive(Debug, Clap)]
pub enum Subcommands {
    /// Initialize the repository in the current directory.
    #[clap(visible_alias = "initialize")]
    #[clap(setting = AppSettings::ColoredHelp, setting = AppSettings::DisableVersionFlag)]
    Init {
        /// The directory in which to initialize a new git repository.
        ///
        /// Defaults to the current working directory.
        directory: Option<PathBuf>,
    },
    /// A selection of useful tools
    #[clap(subcommand)]
    Tools(ToolCommands),
    #[cfg(debug_assertions)]
    Panic,
}

#[derive(Debug, Clap)]
#[clap(setting = AppSettings::ColoredHelp, setting = AppSettings::DisableVersionFlag, setting = AppSettings::SubcommandRequired)]
#[clap(visible_alias = "t")]
pub enum ToolCommands {
    /// Find all repositories in a given directory.
    #[clap(setting = AppSettings::ColoredHelp, setting = AppSettings::DisableVersionFlag)]
    Find {
        /// The directory in which to find all git repositories.
        ///
        /// Defaults to the current working directory.
        root: Option<PathBuf>,
    },
    /// Move all repositories found in a directory into a structure matching their clone URLs.
    #[clap(setting = AppSettings::ColoredHelp, setting = AppSettings::DisableVersionFlag)]
    Organize {
        #[clap(long)]
        /// The operation will be in dry-run mode unless this flag is set.
        execute: bool,

        #[clap(long, short = 'f')]
        /// The directory to use when finding input repositories to move into position.
        ///
        /// Defaults to the current working directory.
        repository_source: Option<PathBuf>,

        #[clap(long, short = 't')]
        /// The directory to which to move repositories found in the repository-source.
        ///
        /// Defaults to the current working directory.
        destination_directory: Option<PathBuf>,
    },
    EstimateHours(EstimateHours),
}

#[derive(Debug, Clap)]
#[clap(
    about = "Estimate hours worked based on a commit history", 
    long_about = "See https://github.com/kimmobrunfeldt/git-hours#how-it-works for details",
    version = clap::crate_version!(),
    visible_alias = "h")
]
#[clap(setting = clap::AppSettings::ColoredHelp)]
pub struct EstimateHours {
    /// The directory containing a '.git/' folder.
    #[clap(parse(from_os_str))]
    #[clap(validator_os = validator::is_repo)]
    #[clap(default_value = ".")]
    pub working_dir: PathBuf,
    /// The name of the ref like 'HEAD' or 'main' at which to start iterating the commit graph.
    #[clap(default_value("HEAD"))]
    pub refname: OsString,
    /// Show personally identifiable information before the summary. Includes names and email addresses.
    #[clap(short = 'p', long)]
    pub show_pii: bool,
    /// Omit unifying identities by name and email which can lead to the same author appear multiple times
    /// due to using different names or email addresses.
    #[clap(short = 'i', long)]
    pub omit_unify_identities: bool,
}

mod validator {
    use std::{ffi::OsStr, path::PathBuf};

    use anyhow::Context;

    fn is_repo_inner(dir: &OsStr) -> anyhow::Result<()> {
        let git_dir = PathBuf::from(dir).join(".git");
        let p = git_dir
            .canonicalize()
            .with_context(|| format!("Could not canonicalize git repository at '{}'", git_dir.display()))?;
        if p.extension().unwrap_or_default() == "git"
            || p.file_name().unwrap_or_default() == ".git"
            || p.join("HEAD").is_file()
        {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Path '{}' needs to be a directory containing '.git/'",
                p.display()
            ))
        }
    }

    pub fn is_repo(dir: &OsStr) -> Result<(), String> {
        is_repo_inner(dir).map_err(|err| format!("{:#}", err))
    }
}
