use clap::{Args, Parser, Subcommand, command};
use semver::Version;

use crate::{commands::CacheAction, spc};

#[derive(Parser)]
#[command(name = "spc-utils")]
#[command(about = "CLI tool for managing Static PHP CLI versions")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Subcommand)]
pub enum Commands {
    #[command(
        about = "Fetch the latest Static PHP CLI version",
        after_help = "Examples:\n  spc-utils latest\n  spc-utils latest -C bulk\n  spc-utils latest -C common -V 8.4\n  spc-utils latest --no-cache"
    )]
    Latest(LatestArgs),

    #[command(
        about = "Check if a given version is the latest",
        after_help = "Examples:\n  spc-utils check-update -V 8.4.10\n  spc-utils check-update -C common -V 8.4.10\n  spc-utils check-update -V 8.4.10 --no-cache"
    )]
    CheckUpdate(CheckUpdateArgs),

    #[command(
        about = "Download a Static PHP CLI binary",
        after_help = "Examples:\n  spc-utils download -o php\n  spc-utils download -C bulk -V 8.4.10 -o php\n  spc-utils download -C common -V 8.4 -O linux -A x86_64 -o ./php-binary\n  spc-utils download --no-cache -o php"
    )]
    Download(DownloadArgs),

    #[command(
        about = "List versions available for download",
        after_help = "Examples:\n  spc-utils list\n  spc-utils list -C common\n  spc-utils list -C common -V 8.4\n  spc-utils list -C common -O linux -A x86_64 -B cli\n  spc-utils list --no-cache"
    )]
    List(ListArgs),

    #[command(
        about = "Manage the local response cache",
        after_help = "Examples:\n  spc-utils cache list\n  spc-utils cache clear\n  spc-utils cache clear -C bulk\n  spc-utils cache path"
    )]
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },

    #[command(about = "Show usage examples for all commands")]
    Examples,
}

#[derive(Args, Clone)]
pub struct DownloadArgs {
    #[arg(short = 'C', long, value_enum)]
    pub category: Option<spc::BuildCategory>,

    #[arg(short = 'V', long, value_parser = validate_version)]
    pub version: Option<Version>,

    #[arg(short = 'O', value_parser = spc::SPC_OS_OPTIONS)]
    pub os: Option<String>,

    #[arg(short = 'A', long, value_parser = spc::SPC_ARCH_OPTIONS)]
    pub arch: Option<String>,

    #[arg(short = 'B', long, value_parser = validate_build_type)]
    pub build_type: Option<String>,

    #[arg(short = 'o', long, help = "Output file path")]
    pub output: String,

    #[arg(long, help = "Skip cache and fetch fresh data")]
    pub no_cache: bool,
}

#[derive(Args, Clone)]
pub struct CheckUpdateArgs {
    #[arg(short = 'C', long, value_enum)]
    pub category: Option<spc::BuildCategory>,

    #[arg(short = 'V', long, value_parser = validate_version)]
    pub version: Version,

    #[arg(long, help = "Skip cache and fetch fresh data")]
    pub no_cache: bool,
}

#[derive(Args, Clone)]
pub struct LatestArgs {
    #[arg(short = 'C', long, value_enum)]
    pub category: Option<spc::BuildCategory>,

    #[arg(short = 'V', long, value_parser = validate_version)]
    pub version: Option<Version>,

    #[arg(short = 'O', value_parser = spc::SPC_OS_OPTIONS)]
    pub os: Option<String>,

    #[arg(short = 'A', long, value_parser = spc::SPC_ARCH_OPTIONS)]
    pub arch: Option<String>,

    #[arg(short = 'B', long, value_parser = validate_build_type)]
    pub build_type: Option<String>,

    #[arg(long, help = "Skip cache and fetch fresh data")]
    pub no_cache: bool,
}

#[derive(Args, Clone)]
pub struct ListArgs {
    #[arg(short = 'C', long, value_enum)]
    pub category: Option<spc::BuildCategory>,

    #[arg(short = 'V', long, value_parser = validate_version)]
    pub version: Option<Version>,

    #[arg(short = 'O', value_parser = spc::SPC_OS_OPTIONS)]
    pub os: Option<String>,

    #[arg(short = 'A', long, value_parser = spc::SPC_ARCH_OPTIONS)]
    pub arch: Option<String>,

    #[arg(short = 'B', long, value_parser = validate_build_type)]
    pub build_type: Option<String>,

    #[arg(long, help = "Skip cache and fetch fresh data")]
    pub no_cache: bool,
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
