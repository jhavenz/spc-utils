use semver::Version;

use crate::spc::{Api, ApiOptions, BuildCategory};

pub fn run_latest(
    category: Option<BuildCategory>,
    version: Option<Version>,
    os: Option<String>,
    arch: Option<String>,
    build_type: Option<String>,
    no_cache: bool,
) {
    let options = ApiOptions::new(category, version, os, arch, build_type);
    let api = Api::new(options).with_no_cache(no_cache);
    let (latest_version, from_cache) = api.fetch_latest_version();

    if from_cache {
        println!("Latest Version: {} (cached)", latest_version);
    } else {
        println!("Latest Version: {}", latest_version);
    }
}
