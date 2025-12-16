use clap::Parser;

mod cli;
mod commands;
mod spc;

use crate::{cli::{Cli, Commands}, spc::Cache};

fn main() {
    let app = Cli::parse();
    let ctx = AppContext::new();

    match app.command {
        Commands::Examples => crate::commands::examples::run(),
        Commands::List(args) => crate::commands::list::run(&ctx, args),
        Commands::Latest(args) => crate::commands::latest::run(&ctx, args),
        Commands::Download(args) => crate::commands::download::run(&ctx, args),
        Commands::Cache { action } => crate::commands::cache::run(&ctx, action),
        Commands::CheckUpdate(args) => crate::commands::check_update::run(&ctx, args),
    }
}

pub struct AppContext {
    pub cache: Cache,
    pub active_os: &'static str,
    pub active_arch: &'static str,
}

impl AppContext {
    pub fn new() -> Self {
        let active_os = std::env::consts::OS;
        let active_arch = std::env::consts::ARCH;

        if !spc::SPC_OS_OPTIONS.contains(&active_os) {
            panic!("Your OS {} is not supported", active_os)
        }

        AppContext {
            cache: Cache::new(),
            active_os,
            active_arch,
        }
    }
}