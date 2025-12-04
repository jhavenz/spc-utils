use semver::Version;

use crate::spc::{Api, ApiOptions, BuildCategory};

pub fn run_check_update(category: Option<BuildCategory>, version: Version, no_cache: bool) {
    let options = ApiOptions::new(category, Some(version.clone()), None, None, None);
    let api = Api::new(options).with_no_cache(no_cache);
    let (latest_version, from_cache) = api.fetch_latest_version();

    let cached_marker = if from_cache { " (cached)" } else { "" };
    if version == latest_version {
        println!("You have the latest version: {}{}", version, cached_marker);
    } else {
        println!(
            "Update available: {} -> {}{}",
            version, latest_version, cached_marker
        );
        println!("  {}", api.download_url(&latest_version));
    }
}
