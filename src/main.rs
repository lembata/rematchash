use clap::Parser;
use md5;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

///Simple tool to delete files based on a hash
#[derive(Parser)]
#[command(author, version, about, long_about)]
struct Cli {
    ///The directory to search in
    path: std::path::PathBuf,

    ///The hash to search for
    #[arg(short = 'H', long)]
    hash: String,

    ///Ignore symlinks
    #[arg(short = 's', long, default_value = "false")]
    ignore_symlinks: bool,

    ///Scan subdirectories
    #[arg(short, long, default_value = "false")]
    recursive: bool,

    ///Ask for confirmation before deleting for each file
    #[arg(short, long, default_value = "false")]
    interactive: bool,

    ///Output more information
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

fn main() {
    let args = Cli::parse();

    if !Path::new(&args.path).exists() || !Path::new(&args.path).is_dir(){
        println!("Directory does not exist: {}", args.path.display());
        return;
    }

    if !is_valid_hash(&args.hash) {
        println!("Invalid hash: {}", &args.hash);
        return;
    }

    let mut handled_paths: Vec<PathBuf> = Vec::new();
    let affected_files = scan_directory(&args.path, &args, &mut handled_paths);
    println!("Deleted {} file(s)", affected_files);
}

fn scan_directory(path: &PathBuf, cli: &Cli, handled_paths: &mut Vec<PathBuf>) -> i32 {
    let mut count = 0;

    if cli.verbose {
        println!("Scanning directory: {}", path.display());
    }

    handled_paths.push(path.clone());

    std::fs::read_dir(path).unwrap().for_each(|entry| {
        let entry = entry.unwrap();
        let entry_path = entry.path();

        if entry_path.is_symlink() {
            if cli.ignore_symlinks {
                if cli.verbose {
                    println!("Ignoring symlink: {}", entry_path.display());
                }
                return;
            }
        }

        let entry_path = if entry_path.is_symlink() {
            if cli.verbose {
                println!("Resolving symlink: {} -> {}", entry_path.display(), entry_path.read_link().unwrap().display());
            }
            fs::read_link(entry_path).unwrap()
        } else {
            entry_path
        };

        if handled_paths.contains(&entry_path) {
            if cli.verbose {
                println!("Ignoring duplicate: {}", entry_path.display());
            }
            return;
        }

        handled_paths.push(entry_path.clone());

        if entry_path.is_dir() {

            if cli.recursive {
                count += scan_directory(&entry.path(), cli, handled_paths);
            }
        } else {
            let hash = md5::compute(fs::read(&entry.path()).unwrap());
            let hash = format!("{:x}", hash);

            if cli.verbose {
                println!("{}: {}", entry.path().display(), hash);
            }

            if cli.hash.contains(&hash) {
                if cli.interactive {
                    if !confirmation_promt(&entry.path()) {
                        return;
                    }
                }

                if let Err(e) = fs::remove_file(&entry.path()) {
                    println!("Failed to delete file: {}", e);
                    return;
                } else {
                    if cli.verbose {
                        println!("Deleted file: {}", entry.path().display());
                    }
                }

                count += 1;
            }
        }
    });

    count
}

fn confirmation_promt(path: &PathBuf) -> bool {
    print!("Delete file {} [y/N]: ", path.display());
    std::io::stdout().flush().expect("flush failed!");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    input.trim().to_lowercase() == "y"
}

fn is_valid_hash(hash: &String) -> bool {
    hash.len() == 32
        && hash.chars().all(|c| c.is_digit(16))
}

