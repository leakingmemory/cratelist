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
    #[arg(required_unless_present = "show_embedded_licenses")]
    lock_path: Option<String>,

    /// Delete all listed crates from the specified directory
    #[arg(short, long)]
    delete: Option<PathBuf>,

    /// List the licenses of all dependencies (requires directory path)
    #[arg(short, long)]
    licenses: Option<PathBuf>,

    /// Show the contents of license files for all dependencies (requires directory path)
    #[arg(short = 'C', long)]
    license_contents: Option<PathBuf>,

    /// Print the embedded licenses of the project and its dependencies
    #[arg(short = 'L', long)]
    show_embedded_licenses: bool,
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

    if args.show_embedded_licenses {
        println!("PROJECT LICENSE");
        println!("===============");
        println!("{}", include_str!("../LICENSE"));
        println!("\nDEPENDENCY LICENSES");
        println!("===================");
        println!("{}", include_str!("../DEPENDENCIES_LICENSE"));
        return;
    }

    let lock_path = args.lock_path.as_ref().unwrap();
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
    } else if let Some(crates_dir) = args.licenses.or(args.license_contents.clone()) {
        let show_contents = args.license_contents.is_some();
        if !crates_dir.is_dir() {
            eprintln!("Error: {} is not a directory.", crates_dir.display());
            process::exit(1);
        }

        for pkg in packages {
            if pkg.source.is_none() {
                continue;
            }

            let mut license = None;
            let mut license_files = Vec::new(); // Stores (filename, content)
            let patterns = vec![
                format!("{}-{}", pkg.name, pkg.version),
                pkg.name.clone(),
            ];

            for pattern in patterns {
                let crate_dir = crates_dir.join(&pattern);
                let toml_path = crate_dir.join("Cargo.toml");
                if toml_path.exists() {
                    if let Ok(toml_content) = fs::read_to_string(&toml_path) {
                        if let Ok(cargo_toml) = toml::from_str::<CargoToml>(&toml_content) {
                            license = cargo_toml.package.license.clone().or(cargo_toml.package.license_file.clone());
                            if show_contents {
                                let mut files_to_try = Vec::new();
                                if let Some(ref lfile) = cargo_toml.package.license_file {
                                    files_to_try.push(lfile.clone());
                                }
                                for common in &[
                                    "LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "COPYING", "UNLICENSE",
                                    "COPYRIGHT", "COPYRIGHT-MIT", "COPYRIGHT-APACHE", "LICENCE", "LICENSE-0BSD",
                                    "license-apache-2.0", "license-mit", "NOTICE", "NOTICES"
                                ] {
                                    files_to_try.push(common.to_string());
                                    files_to_try.push(common.to_lowercase());
                                    for ext in &["txt", "md", "markdown", "license"] {
                                        files_to_try.push(format!("{}.{}", common, ext));
                                        files_to_try.push(format!("{}.{}", common.to_lowercase(), ext));
                                    }
                                }

                                for lfile in files_to_try {
                                    let lpath = crate_dir.join(&lfile);
                                    if lpath.exists() {
                                        if let Ok(content) = fs::read_to_string(&lpath) {
                                            let basename = if let Some(dot_idx) = lfile.rfind('.') {
                                                &lfile[..dot_idx]
                                            } else {
                                                &lfile
                                            };

                                            if !license_files.iter().any(|(f, _): &(String, String)| {
                                                let existing_basename = if let Some(dot_idx) = f.rfind('.') {
                                                    &f[..dot_idx]
                                                } else {
                                                    f
                                                };
                                                existing_basename == basename
                                            }) {
                                                license_files.push((lfile, content));
                                            }
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }

            if license.is_none() || (show_contents && license_files.is_empty()) {
                // Try to find a .crate file and extract Cargo.toml
                let crate_name = format!("{}-{}.crate", pkg.name, pkg.version);
                let crate_path = crates_dir.join(&crate_name);
                if crate_path.exists() {
                    // Try several possible paths inside the tarball
                    let paths_to_try = vec![
                        format!("{}-{}", pkg.name, pkg.version),
                        pkg.name.clone(),
                        "".to_string(),
                    ];

                    let mut last_tar_error = None;
                    for internal_prefix in paths_to_try {
                        let internal_path = if internal_prefix.is_empty() { "Cargo.toml".to_string() } else { format!("{}/Cargo.toml", internal_prefix) };
                        let output = process::Command::new("tar")
                            .args(&["-zOxf", crate_path.to_str().unwrap(), &internal_path])
                            .output();
                        match output {
                            Ok(output) if output.status.success() => {
                                if let Ok(toml_content) = String::from_utf8(output.stdout) {
                                    if let Ok(cargo_toml) = toml::from_str::<CargoToml>(&toml_content) {
                                        license = cargo_toml.package.license.clone().or(cargo_toml.package.license_file.clone());
                                        if show_contents {
                                            let mut files_to_try = Vec::new();
                                            if let Some(ref lfile) = cargo_toml.package.license_file {
                                                files_to_try.push(lfile.clone());
                                            }
                                            for common in &[
                                                "LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "COPYING", "UNLICENSE",
                                                "COPYRIGHT", "COPYRIGHT-MIT", "COPYRIGHT-APACHE", "LICENCE", "LICENSE-0BSD",
                                                "license-apache-2.0", "license-mit", "NOTICE", "NOTICES"
                                            ] {
                                                files_to_try.push(common.to_string());
                                                files_to_try.push(common.to_lowercase());
                                                for ext in &["txt", "md", "markdown", "license"] {
                                                    files_to_try.push(format!("{}.{}", common, ext));
                                                    files_to_try.push(format!("{}.{}", common.to_lowercase(), ext));
                                                }
                                            }

                                            for lfile in files_to_try {
                                                let full_internal_path = if internal_prefix.is_empty() { lfile.clone() } else { format!("{}/{}", internal_prefix, lfile) };
                                                let loutput = process::Command::new("tar")
                                                    .args(&["-zOxf", crate_path.to_str().unwrap(), &full_internal_path])
                                                    .output();
                                                if let Ok(loutput) = loutput {
                                                    if loutput.status.success() {
                                                        if let Ok(content) = String::from_utf8(loutput.stdout) {
                                                            let basename = if let Some(dot_idx) = lfile.rfind('.') {
                                                                &lfile[..dot_idx]
                                                            } else {
                                                                &lfile
                                                            };

                                                            if !license_files.iter().any(|(f, _): &(String, String)| {
                                                                let existing_basename = if let Some(dot_idx) = f.rfind('.') {
                                                                    &f[..dot_idx]
                                                                } else {
                                                                    f
                                                                };
                                                                existing_basename == basename
                                                            }) {
                                                                license_files.push((lfile, content));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
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

            if show_contents {
                if !license_files.is_empty() {
                    for (filename, content) in license_files {
                        let content: String = content;
                        if let Err(e) = writeln!(io::stdout(), "================================================================================") {
                            if e.kind() == io::ErrorKind::BrokenPipe { break; }
                        }
                        if let Err(e) = writeln!(io::stdout(), "{}@{}: {} ({})", pkg.name, pkg.version, license.as_deref().unwrap_or("Unknown"), filename) {
                            if e.kind() == io::ErrorKind::BrokenPipe { break; }
                        }
                        if let Err(e) = writeln!(io::stdout(), "--------------------------------------------------------------------------------") {
                            if e.kind() == io::ErrorKind::BrokenPipe { break; }
                        }
                        if let Err(e) = writeln!(io::stdout(), "{}", content.trim()) {
                            if e.kind() == io::ErrorKind::BrokenPipe { break; }
                        }
                        if let Err(e) = writeln!(io::stdout(), "================================================================================") {
                            if e.kind() == io::ErrorKind::BrokenPipe { break; }
                        }
                        if let Err(e) = writeln!(io::stdout(), "") {
                            if e.kind() == io::ErrorKind::BrokenPipe { break; }
                        }
                    }
                } else {
                    if let Err(e) = writeln!(io::stderr(), "Warning: Could not find license file for {}@{}", pkg.name, pkg.version) {
                         if e.kind() == io::ErrorKind::BrokenPipe { break; }
                    }
                }
            } else {
                if let Err(e) = writeln!(io::stdout(), "{}@{}: {}", pkg.name, pkg.version, license.unwrap_or_else(|| "Unknown".to_string())) {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        break;
                    }
                    eprintln!("Error writing to stdout: {}", e);
                    process::exit(1);
                }
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
