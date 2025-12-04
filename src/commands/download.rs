use semver::Version;

use crate::spc::{Api, ApiOptions, BuildCategory};

pub fn run_download(
    category: Option<BuildCategory>,
    version: Option<Version>,
    os: Option<String>,
    arch: Option<String>,
    build_type: Option<String>,
    output: String,
    no_cache: bool,
) {
    let options = ApiOptions::new(category, version, os, arch, build_type);
    let api = Api::new(options).with_no_cache(no_cache);

    match api.download(&output) {
        Ok(()) => println!("Download complete!"),
        Err(e) => eprintln!("Download failed: {}", e),
    }
}
