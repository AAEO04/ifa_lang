use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct InstallerArgs {
    /// Run in headless mode (no GUI)
    #[arg(long)]
    pub headless: bool,

    /// Automatically accept all defaults/prompts
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Custom installation directory
    #[arg(short, long)]
    pub dir: Option<PathBuf>,

    /// Uninstall Ifá-Lang
    #[arg(long)]
    pub uninstall: bool,
}
