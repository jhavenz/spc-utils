use crate::{
    AppContext,
    cli::CheckUpdateArgs,
    spc::{Api, ApiOptions},
};

pub fn run(ctx: &AppContext, args: CheckUpdateArgs) {
    let options = ApiOptions::new(
        args.category.clone(),
        Some(args.version.clone()),
        None,
        None,
        None,
    );
    let api = Api::new(ctx.cache.clone(), options).with_no_cache(args.no_cache);
    let (latest_version, from_cache) = api.fetch_latest_version();

    let cached_marker = if from_cache { " (cached)" } else { "" };
    if args.version == latest_version {
        println!(
            "You have the latest version: {}{}",
            args.version, cached_marker
        );
    } else {
        println!(
            "Update available: {} -> {}{}",
            args.version, latest_version, cached_marker
        );
        println!("  {}", api.download_url(&latest_version));
    }
}
