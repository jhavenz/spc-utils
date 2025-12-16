use chrono::{DateTime, Local};
use clap::Subcommand;
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};

use crate::{AppContext, spc::BuildCategory};

#[derive(Clone, Subcommand)]
pub enum CacheAction {
    #[command(about = "List all cached files with details")]
    List,
    #[command(about = "Clear cached files")]
    Clear {
        #[arg(short = 'C', long, value_enum, help = "Clear only a specific category")]
        category: Option<BuildCategory>,
    },
    #[command(about = "Print the cache directory path")]
    Path,
}

pub fn run(ctx: &AppContext, action: CacheAction) {
    let cache = &ctx.cache;

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
