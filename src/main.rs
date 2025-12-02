use core::panic;

use clap::{Parser, Subcommand};
use semver::Version;

mod spc;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let active_os = &std::env::consts::OS;

    if !spc::SPC_OS_OPTIONS.contains(active_os) {
        panic!("Your OS {} is not supported", active_os)
    }

    let app = Cli::parse();

    match app.command {
        Commands::Latest {
            category,
            version,
            os,
            arch,
            build_type,
            no_cache,
        } => {
            let options = spc::ApiOptions::new(category, version, os, arch, build_type);

            let api = spc::Api::new(options).with_no_cache(no_cache);
            let latest_version = api.fetch_latest_version();

            println!("Latest Version: {}", latest_version);
        }
        Commands::CheckUpdate {
            category,
            version,
            no_cache,
        } => {
            let options = spc::ApiOptions::new(category, Some(version.clone()), None, None, None);

            let api = spc::Api::new(options).with_no_cache(no_cache);
            let latest_version = api.fetch_latest_version();

            if version == latest_version {
                println!("✓ You have the latest version: {}", version);
            } else {
                println!("✗ Update available: {} → {}", version, latest_version);
            }
        }
        Commands::Download {
            category,
            version,
            os,
            arch,
            build_type,
            output,
            no_cache,
        } => {
            let options = spc::ApiOptions::new(category, version, os, arch, build_type);

            let api = spc::Api::new(options).with_no_cache(no_cache);

            match api.download(&output) {
                Ok(_) => println!("Download complete!"),
                Err(e) => eprintln!("Download failed: {}", e),
            }
        }
    }
}

#[derive(Clone, Subcommand)]
enum Commands {
    #[command(
        about = "Fetch the latest Static PHP CLI version",
        after_help = "Examples:
        spc-version latest
        spc-version latest -C bulk
        spc-version latest -C common -V 8.4
        spc-version latest --no-cache
    "
    )]
    Latest {
        #[arg(short = 'C', long, value_enum)]
        category: Option<spc::BuildCategory>,

        #[arg(short = 'V', long, value_parser = validate_version)]
        version: Option<Version>,

        #[arg(short = 'O', value_parser = spc::SPC_OS_OPTIONS)]
        os: Option<String>,

        #[arg(short = 'A', long, value_parser = spc::SPC_ARCH_OPTIONS)]
        arch: Option<String>,

        #[arg(short = 'B', long, value_parser = validate_build_type)]
        build_type: Option<String>,

        #[arg(long, help = "Skip cache and fetch fresh data")]
        no_cache: bool,
    },
    #[command(
        about = "Check if a given version is the latest",
        after_help = "Examples:
        spc-version check-update -v 8.4.10
        spc-version check-update -C common -v 8.4.10
        spc-version check-update -v 8.4.10 --no-cache
    "
    )]
    CheckUpdate {
        #[arg(short = 'C', long, value_enum)]
        category: Option<spc::BuildCategory>,

        #[arg(short, long)]
        version: Version,

        #[arg(long, help = "Skip cache and fetch fresh data")]
        no_cache: bool,
    },
    #[command(
        about = "Download a Static PHP CLI binary",
        after_help = "Examples:
        spc-version download -o php
        spc-version download -C bulk -V 8.4.10 -o php
        spc-version download -C common -V 8.4 -O linux -A x86_64 -o ./php-binary
        spc-version download --no-cache -o php
    "
    )]
    Download {
        #[arg(short = 'C', long, value_enum)]
        category: Option<spc::BuildCategory>,

        #[arg(short = 'V', long, value_parser = validate_version)]
        version: Option<Version>,

        #[arg(short = 'O', value_parser = spc::SPC_OS_OPTIONS)]
        os: Option<String>,

        #[arg(short = 'A', long, value_parser = spc::SPC_ARCH_OPTIONS)]
        arch: Option<String>,

        #[arg(short = 'B', long, value_parser = validate_build_type)]
        build_type: Option<String>,

        #[arg(short = 'o', long, help = "Output file path")]
        output: String,

        #[arg(long, help = "Skip cache and fetch fresh data")]
        no_cache: bool,
    },
}

fn validate_version(input: &str) -> Result<Version, String> {
    if let Ok(proper_version) = Version::parse(input) {
        return Ok(proper_version);
    }

    let parts: Vec<&str> = input.split(".").map(|s| s.trim()).collect();

    let maj = parts.get(0).unwrap_or(&"8");
    let min = parts.get(1).unwrap_or(&"0");
    let patch = parts.get(2).unwrap_or(&"0");

    Version::parse(&format!("{}.{}.{}", maj, min, patch))
        .map_err(|e| format!("Invalid version '{}': {}", input, e))
}

fn validate_build_type(input: &str) -> Result<String, String> {
    if !spc::SPC_PHP_BUILD_TYPE_OPTIONS.contains(&input) {
        return Err(format!("Invalid build type: {}", input));
    }

    let has_fpm_arg = std::env::args()
        .collect::<Vec<String>>()
        .contains(&String::from("fpm"));

    if has_fpm_arg && (input == "win-min" || input == "win-max") {
        return Err(format!(
            "Build type '{}' does not support the 'fpm' category",
            input
        ));
    }

    Ok(input.to_string())
}
