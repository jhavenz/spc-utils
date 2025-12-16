use semver::Version;

use crate::{AppContext, cli::ListArgs, spc::{Api, ApiOptions, BuildCategory}};

pub fn run(ctx: &AppContext, args: ListArgs) {
	let options = ApiOptions::new(args.category, args.version, args.os, args.arch, args.build_type);

	let os_needle = options.os();
	let arch_needle = options.arch();
	let category = options.category();
	let build_type_needle = options.build_type();
	let version_bound = options.version_bound().cloned();

	let api = Api::new(ctx.cache.clone(), options).with_no_cache(args.no_cache);

	let (data, _) = match api.fetch_versions() {
		Ok(v) => v,
		Err(e) => {
			eprintln!("Failed to fetch versions: {}", e);
			return;
		}
	};

	let mut versions: Vec<Version> = data
		.into_iter()
		.filter(|resp| {
			let version_match = if let Some(v) = resp.version() {
				if let Some(bound) = version_bound.as_ref() {
					v.major == bound.major && v.minor == bound.minor
				} else {
					true
				}
			} else {
				false
			};

			let name_match = match category {
				BuildCategory::WinMin | BuildCategory::WinMax => {
					resp.name.contains(&build_type_needle) && resp.name.ends_with("-win.zip")
				}
				_ => {
					resp.name.contains(&os_needle)
						&& resp.name.contains(&arch_needle)
						&& resp.name.contains(&build_type_needle)
				}
			};

			version_match && name_match
		})
		.filter_map(|resp| resp.version())
		.collect();

	versions.sort();
	versions.dedup();
	versions.sort_by(|a, b| b.cmp(a));

	for v in versions {
		println!("{}", v);
	}
}
