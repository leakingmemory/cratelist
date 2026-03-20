use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the Cargo.lock file
    #[arg(required = true)]
    lock_path: String,

    /// Delete all listed crates from the specified directory
    #[arg(short, long)]
    delete: Option<PathBuf>,
}

#[derive(Deserialize)]
struct LockFile {
    #[serde(rename = "package")]
    packages: Option<Vec<Package>>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
    source: Option<String>,
}

fn main() {
    let args = Args::parse();

    let lock_path = &args.lock_path;
    let content = fs::read_to_string(lock_path).unwrap_or_else(|err| {
        eprintln!("Error reading file {}: {}", lock_path, err);
        process::exit(1);
    });

    let lock: LockFile = toml::from_str(&content).unwrap_or_else(|err| {
        eprintln!("Error parsing Cargo.lock: {}", err);
        process::exit(1);
    });

    let packages = match lock.packages {
        Some(pkgs) => pkgs,
        None => {
            println!("No packages found in Cargo.lock.");
            return;
        }
    };

    if let Some(delete_dir) = args.delete {
        if !delete_dir.is_dir() {
            eprintln!("Error: {} is not a directory.", delete_dir.display());
            process::exit(1);
        }

        for pkg in packages {
            if pkg.source.is_none() {
                continue;
            }
            // Usually crates in vendor/ or registry/src/ follow name-version or name
            // Let's check both patterns commonly used by Cargo
            let patterns = vec![
                format!("{}-{}.crate", pkg.name, pkg.version), // registry/src/...
                pkg.name.clone(),                       // vendor/
            ];

            for pattern in patterns {
                let target = delete_dir.join(&pattern);
                if target.exists() {
                    println!("Deleting {}...", target.display());
                    if let Err(err) = fs::remove_dir_all(&target) {
                        // If it's a file, try remove_file
                        if let Err(file_err) = fs::remove_file(&target) {
                            eprintln!("Failed to delete {}: {} (also tried file: {})", target.display(), err, file_err);
                        } else {
                            println!("Deleted file {}.", target.display());
                        }
                    } else {
                        println!("Deleted directory {}.", target.display());
                    }
                }
            }
        }
    } else {
        for pkg in packages {
            if pkg.source.is_some() {
                println!("{}@{}", pkg.name, pkg.version);
            }
        }
    }
}
