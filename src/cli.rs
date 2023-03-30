use clap::Parser;

///Simple tool to delete files based on a hash
#[derive(Parser)]
#[command(author, version, about, long_about)]
pub struct Cli {
    ///The directory to search in
    pub path: std::path::PathBuf,

    ///The hash to search for
    #[arg(short = 'H', long)]
    pub hash: String,

    ///Ignore symlinks
    #[arg(short = 's', long, default_value = "false")]
    pub ignore_symlinks: bool,

    ///Scan subdirectories
    #[arg(short, long, default_value = "false")]
    pub recursive: bool,

    ///Ask for confirmation before deleting for each file
    #[arg(short, long, default_value = "false")]
    pub interactive: bool,

    ///Output more information
    #[arg(short, long, default_value = "false")]
    pub verbose: bool,
}
