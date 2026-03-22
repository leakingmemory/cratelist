use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::io::{self, Write};
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

    /// List the licenses of all dependencies (requires directory path)
    #[arg(short, long)]
    licenses: Option<PathBuf>,
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

#[derive(Deserialize)]
struct CargoToml {
    package: PackageMetadata,
}

#[derive(Deserialize)]
struct PackageMetadata {
    license: Option<String>,
    #[serde(rename = "license-file")]
    license_file: Option<String>,
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
        Some(mut pkgs) => {
            pkgs.sort_by(|a, b| a.name.cmp(&b.name));
            pkgs
        }
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
            // Usually crates in registry/cache/ follow name-version.crate
            // And extracted crates in registry/src/ or vendor/ follow name-version or name
            let patterns = vec![
                format!("{}-{}", pkg.name, pkg.version),      // registry/src/...
                format!("{}-{}.crate", pkg.name, pkg.version), // registry/cache/...
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
    } else if let Some(crates_dir) = args.licenses {
        if !crates_dir.is_dir() {
            eprintln!("Error: {} is not a directory.", crates_dir.display());
            process::exit(1);
        }

        for pkg in packages {
            if pkg.source.is_none() {
                continue;
            }

            let mut license = None;
            let patterns = vec![
                format!("{}-{}", pkg.name, pkg.version),
                pkg.name.clone(),
            ];

            for pattern in patterns {
                let toml_path = crates_dir.join(&pattern).join("Cargo.toml");
                if toml_path.exists() {
                    if let Ok(toml_content) = fs::read_to_string(&toml_path) {
                        if let Ok(cargo_toml) = toml::from_str::<CargoToml>(&toml_content) {
                            license = cargo_toml.package.license.or(cargo_toml.package.license_file);
                            break;
                        }
                    }
                }
            }

            if license.is_none() {
                // Try to find a .crate file and extract Cargo.toml
                let crate_name = format!("{}-{}.crate", pkg.name, pkg.version);
                let crate_path = crates_dir.join(&crate_name);
                if crate_path.exists() {
                    // Try several possible paths inside the tarball
                    let paths_to_try = vec![
                        format!("{}-{}/Cargo.toml", pkg.name, pkg.version),
                        format!("{}/Cargo.toml", pkg.name),
                        "Cargo.toml".to_string(),
                    ];

                    let mut last_tar_error = None;
                    for internal_path in paths_to_try {
                        let output = process::Command::new("tar")
                            .args(&["-zOxf", crate_path.to_str().unwrap(), &internal_path])
                            .output();
                        match output {
                            Ok(output) if output.status.success() => {
                                if let Ok(toml_content) = String::from_utf8(output.stdout) {
                                    if let Ok(cargo_toml) = toml::from_str::<CargoToml>(&toml_content) {
                                        license = cargo_toml.package.license.or(cargo_toml.package.license_file);
                                        if license.is_some() {
                                            break;
                                        }
                                    }
                                }
                            }
                            Ok(output) => {
                                last_tar_error = Some(String::from_utf8_lossy(&output.stderr).to_string());
                            }
                            Err(e) => {
                                last_tar_error = Some(e.to_string());
                            }
                        }
                    }

                    if license.is_none() && last_tar_error.is_some() {
                        eprintln!("Error extracting {}: {}", crate_name, last_tar_error.unwrap().trim());
                    }
                }
            }

            if let Err(e) = writeln!(io::stdout(), "{}@{}: {}", pkg.name, pkg.version, license.unwrap_or_else(|| "Unknown".to_string())) {
                if e.kind() == io::ErrorKind::BrokenPipe {
                    break;
                }
                eprintln!("Error writing to stdout: {}", e);
                process::exit(1);
            }
        }
    } else {
        for pkg in packages {
            if pkg.source.is_some() {
                if let Err(e) = writeln!(io::stdout(), "{}@{}", pkg.name, pkg.version) {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        break;
                    }
                    eprintln!("Error writing to stdout: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
