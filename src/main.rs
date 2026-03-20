use serde::Deserialize;
use std::env;
use std::fs;
use std::process;

#[derive(Deserialize)]
struct LockFile {
    #[serde(rename = "package")]
    packages: Option<Vec<Package>>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <Cargo.lock path>", args[0]);
        process::exit(1);
    }

    let lock_path = &args[1];
    let content = fs::read_to_string(lock_path).unwrap_or_else(|err| {
        eprintln!("Error reading file {}: {}", lock_path, err);
        process::exit(1);
    });

    let lock: LockFile = toml::from_str(&content).unwrap_or_else(|err| {
        eprintln!("Error parsing Cargo.lock: {}", err);
        process::exit(1);
    });

    if let Some(packages) = lock.packages {
        for pkg in packages {
            println!("{}@{}", pkg.name, pkg.version);
        }
    }
}
