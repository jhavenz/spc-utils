use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};
use semver::Version;

mod spc;

#[derive(Parser)]
#[command(name = "spc-utils")]
#[command(about = "CLI tool for managing Static PHP CLI versions")]
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
            let (latest_version, from_cache) = api.fetch_latest_version();

            if from_cache {
                println!("Latest Version: {} (cached)", latest_version);
            } else {
                println!("Latest Version: {}", latest_version);
            }
        }
        Commands::CheckUpdate {
            category,
            version,
            no_cache,
        } => {
            let options = spc::ApiOptions::new(category, Some(version.clone()), None, None, None);

            let api = spc::Api::new(options).with_no_cache(no_cache);
            let (latest_version, from_cache) = api.fetch_latest_version();

            let cached_marker = if from_cache { " (cached)" } else { "" };
            if version == latest_version {
                println!(
                    "✓ You have the latest version: {}{}",
                    version, cached_marker
                );
            } else {
                println!(
                    "✗ Update available: {} → {}{}",
                    version, latest_version, cached_marker
                );
                println!("  {}", api.download_url(&latest_version));
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
        Commands::Cache { action } => {
            let cache = spc::Cache::new();

            match action {
                CacheAction::List => {
                    let files = cache.list_cached_files();

                    if files.is_empty() {
                        println!("No cached files found.");
                        println!("Cache directory: {}", cache.cache_dir().display());
                        return;
                    }

                    let mut table = Table::new();
                    table
                        .load_preset(UTF8_FULL)
                        .set_content_arrangement(ContentArrangement::Dynamic)
                        .set_header(vec![
                            Cell::new("Category"),
                            Cell::new("Entries"),
                            Cell::new("Size"),
                            Cell::new("Modified"),
                            Cell::new("Expires"),
                        ]);

                    for file in &files {
                        table.add_row(vec![
                            Cell::new(file.category.to_string()),
                            Cell::new(file.entry_count.to_string()),
                            Cell::new(format_size(file.size)),
                            Cell::new(file.modified.format("%Y-%m-%d %H:%M").to_string()),
                            Cell::new(format_expires(&file.expires)),
                        ]);
                    }

                    println!("{table}");
                    println!("\nCache directory: {}", cache.cache_dir().display());
                }
                CacheAction::Clear { category } => match cache.clear(category.as_ref()) {
                    Ok(count) => {
                        if count == 0 {
                            println!("No cache files to remove.");
                        } else {
                            println!("Removed {} cache file(s).", count);
                        }
                    }
                    Err(e) => eprintln!("Failed to clear cache: {}", e),
                },
                CacheAction::Path => {
                    println!("{}", cache.cache_dir().display());
                }
            }
        }
        Commands::Examples => {
            print_examples();
        }
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_expires(expires: &DateTime<Local>) -> String {
    let now = Local::now();
    if *expires <= now {
        "expired".to_string()
    } else {
        let duration = *expires - now;
        let hours = duration.num_hours();
        let minutes = duration.num_minutes() % 60;
        if hours > 0 {
            format!("in {}h {}m", hours, minutes)
        } else {
            format!("in {}m", minutes)
        }
    }
}

fn print_examples() {
    println!(
        r#"Usage Examples:

  Get the latest version:
    spc-version latest
    spc-version latest -C common -V 8.4

  Check for updates:
    spc-version check-update -v 8.4.10

  Download a binary:
    spc-version download -o php
    spc-version download -C bulk -V 8.4 -o ./php-bin

  Manage cache:
    spc-version cache list
    spc-version cache clear

  Skip cache on any command:
    spc-version latest --no-cache"#
    );
}

#[derive(Clone, Subcommand)]
enum Commands {
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
  spc-utils check-update -v 8.4.10
  spc-utils check-update -C common -v 8.4.10
  spc-utils check-update -v 8.4.10 --no-cache"
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
  spc-version cache list
  spc-version cache clear
  spc-version cache clear -C bulk
  spc-version cache path"
    )]
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
    #[command(about = "Show usage examples for all commands")]
    Examples,
}

#[derive(Clone, Subcommand)]
enum CacheAction {
    #[command(about = "List all cached files with details")]
    List,
    #[command(about = "Clear cached files")]
    Clear {
        #[arg(short = 'C', long, value_enum, help = "Clear only a specific category")]
        category: Option<spc::BuildCategory>,
    },
    #[command(about = "Print the cache directory path")]
    Path,
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
