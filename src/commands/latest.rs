use crate::{AppContext, cli::LatestArgs, spc::{Api, ApiOptions}};

pub fn run(ctx: &AppContext, args: LatestArgs) {
    let options = ApiOptions::new(
        args.category,
        args.version,
        args.os,
        args.arch,
        args.build_type,
    );
    let api = Api::new(ctx.cache.clone(), options).with_no_cache(args.no_cache);
    let (latest_version, from_cache) = api.fetch_latest_version();

    if from_cache {
        println!("Latest Version: {} (cached)", latest_version);
    } else {
        println!("Latest Version: {}", latest_version);
    }
}
