use clap::Parser;
use md5;
use sha1::{Sha1, Digest};
use sha2::Sha256;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use crate::cli::Cli;

mod cli;

#[derive(Debug)]
enum HashType {
    MD5,
    SHA1,
    SHA256
}

fn main() {
    let args = Cli::parse();

    if !Path::new(&args.path).exists() || !Path::new(&args.path).is_dir(){
        println!("Directory does not exist: {}", args.path.display());
        return;
    }

    let hash_type = validate_hash(&args.hash);

    if args.verbose {
        println!("Hash type: {:?}", hash_type);
    }

    if hash_type.is_none() {
        println!("Invalid hash: {}", &args.hash);
        return;
    }

    let hash_type = hash_type.unwrap();

    let mut handled_paths: Vec<PathBuf> = Vec::new();
    let affected_files = scan_directory(&args.path, &args, &hash_type, &mut handled_paths);
    println!("Deleted {} file(s)", affected_files);
}

fn scan_directory(path: &PathBuf, cli: &Cli, hash_type: &HashType, handled_paths: &mut Vec<PathBuf>) -> i32 {
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

        let entry_path = expand_symlink(entry_path, cli.verbose);

        if handled_paths.contains(&entry_path) {
            if cli.verbose {
                println!("Ignoring duplicate: {}", entry_path.display());
            }
            return;
        }

        handled_paths.push(entry_path.clone());

        if entry_path.is_dir() {

            if cli.recursive {
                count += scan_directory(&entry.path(), cli, hash_type, handled_paths);
            }
        } else {
            let hash = get_hash(&entry.path(), hash_type);

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

fn get_hash(path: &PathBuf, hash_type: &HashType) -> String {
    match hash_type {
        HashType::MD5 => get_hash_md5(path),
        HashType::SHA1 => get_hash_sha1(path),
        HashType::SHA256 => get_hash_sha2(path)
    }
}

fn get_hash_md5(path: &PathBuf) -> String {
    let hash = md5::compute(fs::read(path).unwrap());
    format!("{:x}", hash)
}

fn get_hash_sha1(path: &PathBuf) -> String {
    let mut sh = Sha1::default();
    sh.update(fs::read(path).unwrap());

    let hash = sh.finalize();
    format!("{:x}", hash)
}

fn get_hash_sha2(path: &PathBuf) -> String {
    let mut sh = Sha256::default();
    sh.update(fs::read(path).unwrap());

    let hash = sh.finalize();
    format!("{:x}", hash)
}

fn confirmation_promt(path: &PathBuf) -> bool {
    print!("Delete file {} [y/N]: ", path.display());
    std::io::stdout().flush().expect("flush failed!");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    input.trim().to_lowercase() == "y"
}

fn validate_hash(hash: &String) -> Option<HashType> {
    if is_valid_hash_md5(hash) {
        return Some(HashType::MD5);
    }

    if is_valid_hash_sha1(hash) {
        return Some(HashType::SHA1);
    }

    if is_valid_hash_sha2(hash) {
        return Some(HashType::SHA256);
    }

    None
}

fn is_valid_hash_md5(hash: &String) -> bool {
    hash.len() == 32
        && hash.chars().all(|c| c.is_digit(16))
}

fn is_valid_hash_sha1(hash: &String) -> bool {
    hash.len() == 40
        && hash.chars().all(|c| c.is_digit(16))
}

fn is_valid_hash_sha2(hash: &String) -> bool {
    hash.len() == 64
        && hash.chars().all(|c| c.is_digit(16))
}

fn expand_symlink(entry_path :PathBuf, verbose :bool) -> PathBuf {
    if entry_path.is_symlink() {
        if verbose {
            println!("Resolving symlink: {} -> {}", entry_path.display(), entry_path.read_link().unwrap().display());
        }
        return fs::read_link(entry_path).unwrap()
    } else {
        return entry_path
    };
}
