use clap::Parser;
use std::path::PathBuf;
use crate::profiles::Profile;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct InstallerArgs {
    /// Run in headless mode (no GUI)
    #[arg(long)]
    pub headless: bool,

    /// Automatically accept all defaults/prompts
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Installation profile to use
    #[arg(short, long, value_enum, default_value_t = Profile::Standard)]
    pub profile: Profile,

    /// Custom installation directory
    #[arg(short, long)]
    pub dir: Option<PathBuf>,

    /// Uninstall Ifá-Lang
    #[arg(long)]
    pub uninstall: bool,
}
