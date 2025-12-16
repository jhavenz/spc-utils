use crate::{AppContext, cli::DownloadArgs, spc::{Api, ApiOptions}};

pub fn run(ctx: &AppContext, args: DownloadArgs) {
    let options = ApiOptions::new(
        args.category,
        args.version,
        args.os,
        args.arch,
        args.build_type,
    );

    let output = args.output;
    let api = Api::new(ctx.cache.clone(), options).with_no_cache(args.no_cache);

    match api.download(&output) {
        Ok(()) => println!("Download complete!"),
        Err(e) => eprintln!("Download failed: {}", e),
    }
}
