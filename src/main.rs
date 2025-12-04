use clap::{Parser, Subcommand, command};

mod commands;
mod spc;

use commands::{run_cache, run_check_update, run_download, run_examples, run_latest};
use semver::Version;

use crate::commands::CacheAction;

#[derive(Parser)]
#[command(name = "spc-utils")]
#[command(about = "CLI tool for managing Static PHP CLI versions")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Subcommand)]
pub enum Commands {
    #[command(
        about = "Fetch the latest Static PHP CLI version",
        after_help = "Examples:
  spc-utils latest
  spc-utils latest -C bulk
  spc-utils latest -C common -V 8.4
  spc-utils latest --no-cache"
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
  spc-utils check-update -V 8.4.10
  spc-utils check-update -C common -V 8.4.10
  spc-utils check-update -V 8.4.10 --no-cache"
    )]
    CheckUpdate {
        #[arg(short = 'C', long, value_enum)]
        category: Option<spc::BuildCategory>,

        #[arg(short = 'V', long, value_parser = validate_version)]
        version: Version,

        #[arg(long, help = "Skip cache and fetch fresh data")]
        no_cache: bool,
    },
    #[command(
        about = "Download a Static PHP CLI binary",
        after_help = "Examples:
  spc-utils download -o php
  spc-utils download -C bulk -V 8.4.10 -o php
  spc-utils download -C common -V 8.4 -O linux -A x86_64 -o ./php-binary
  spc-utils download --no-cache -o php"
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
    #[command(
        about = "Manage the local response cache",
        after_help = "Examples:
  spc-utils cache list
  spc-utils cache clear
  spc-utils cache clear -C bulk
  spc-utils cache path"
    )]
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
    #[command(about = "Show usage examples for all commands")]
    Examples,
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
        } => run_latest(category, version, os, arch, build_type, no_cache),

        Commands::CheckUpdate {
            category,
            version,
            no_cache,
        } => run_check_update(category, version, no_cache),

        Commands::Download {
            category,
            version,
            os,
            arch,
            build_type,
            output,
            no_cache,
        } => run_download(category, version, os, arch, build_type, output, no_cache),

        Commands::Cache { action } => run_cache(action),

        Commands::Examples => run_examples(),
    }
}

fn validate_version(input: &str) -> Result<Version, String> {
    let version = if let Ok(v) = Version::parse(input) {
        v
    } else {
        let parts: Vec<&str> = input.split('.').map(|s| s.trim()).collect();

        let maj = parts.first().unwrap_or(&"8");
        let min = parts.get(1).unwrap_or(&"0");
        let patch = parts.get(2).unwrap_or(&"0");

        Version::parse(&format!("{}.{}.{}", maj, min, patch))
            .map_err(|e| format!("Invalid version '{}': {}", input, e))?
    };

    if version.major < 8 {
        return Err(format!(
            "Version {} is not supported. \nSPC only provides PHP 8.0.0 and later.",
            version
        ));
    }

    Ok(version)
}

fn validate_build_type(input: &str) -> Result<String, String> {
    if !spc::SPC_PHP_BUILD_TYPE_OPTIONS.contains(&input) {
        return Err(format!("Invalid build type: {}", input));
    }

    let has_fpm_arg = std::env::args().any(|arg| arg == "fpm");

    if has_fpm_arg && (input == "win-min" || input == "win-max") {
        return Err(format!(
            "Build type '{}' does not support the 'fpm' category",
            input
        ));
    }

    Ok(input.to_string())
}
