use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use clap::Parser;
use md5;

///Simple tool to delete files based on a hash
#[derive(Parser)]
#[command(author, version, about, long_about)]
struct Cli {
    ///The directory to search in
    path: std::path::PathBuf,

    ///The hash to search for
    #[arg(short = 'H', long)]
    hash: Vec<String>,

    ///Ignore symlinks
    #[arg(short = 's', long, default_value = "true")]
    ignore_symlinks: bool,

    ///Scan subdirectories
    #[arg(short, long, default_value = "false")]
    recursive: bool,

    ///Ask for confirmation before deleting for each file
    #[arg(short, long, default_value = "false")]
    interactive: bool,
}


fn main() {
    let args = Cli::parse();

    if !Path::new(&args.path).exists() {
        println!("Directory does not exist: {}", args.path.display());
        return;
    }

    let affected_files = scan_directory(&args.path,&args);

    println!("Deleted {} file(s)", affected_files);
}

fn scan_directory(path: &PathBuf, cli: &Cli) -> i32 {
    let mut count = 0;
    std::fs::read_dir(path).unwrap().for_each(|entry| {
        let entry = entry.unwrap();

        if cli.ignore_symlinks && Path::new(&entry.path()).is_symlink() {
            return;
        }
        else if Path::new(&entry.path()).is_dir() {
            if cli.recursive {
                count += scan_directory(&entry.path(), cli);
            }
        }

        if Path::new(&entry.path()).is_file() {
            let hash = md5::compute(fs::read(&entry.path()).unwrap());
            let hash = format!("{:x}", hash);
            if cli.hash.contains(&hash) {

                if cli.interactive {
                    print!("Delete file {} [y/N]: ", entry.path().display());
                    std::io::stdout().flush().expect("flush failed!");
                        //.flush().expect("flush failed!");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    if input.trim() != "y" {
                        return;
                    }
                }

                if let Err(e) = fs::remove_file(&entry.path()) {
                    println!("Failed to delete file: {}", e);
                }
                count += 1;
            }
        }
    });
    count
}
