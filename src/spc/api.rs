use reqwest::blocking;
use semver::Version;
use std::env::consts::{ARCH, OS};

use super::{BuildCategory, Cache, SpcJsonResponse};

pub struct ApiOptions {
    category: Option<BuildCategory>,
    version: Option<Version>,
    os: Option<String>,
    arch: Option<String>,
    build_type: Option<String>,
}

impl ApiOptions {
    pub fn new(
        category: Option<BuildCategory>,
        version: Option<Version>,
        os: Option<String>,
        arch: Option<String>,
        build_type: Option<String>,
    ) -> Self {
        Self {
            category,
            version,
            os,
            arch,
            build_type,
        }
    }

    fn to_url(&self, base_url: &str) -> String {
        format!("{}/{}?format=json", base_url, self.category_path())
    }

    fn to_download_url(&self, base_url: &str) -> String {
        format!("{}/{}/{}", base_url, self.category_path(), self.file_name())
    }

    fn category_path(&self) -> String {
        match self.category() {
            BuildCategory::Bulk => "bulk".to_string(),
            BuildCategory::Common => "common".to_string(),
            BuildCategory::Minimal => "minimal".to_string(),
            BuildCategory::WinMin => "windows/spc-min".to_string(),
            BuildCategory::WinMax => "windows/spc-max".to_string(),
        }
    }

    pub fn category(&self) -> BuildCategory {
        self.category
            .clone()
            .unwrap_or_else(BuildCategory::default_for_os)
    }

    ///
    /// Examples:
    /// winmax, winmin -> php-8.1.29-micro-win.zip, php-8.1.31-cli-win.zip
    /// minimal -> php-8.0.30-cli-linux-x86_64.tar.gz, php-8.0.30-fpm-linux-aarch64.tar.gz, php-8.0.30-micro-linux-aarch64.tar.gz
    /// common -> php-8.0.30-cli-linux-x86_64.tar.gz, php-8.1.23-fpm-linux-x86_64.tar.gz, php-8.1.25-micro-linux-aarch64.tar.gz
    /// bulk -> php-8.0.30-cli-linux-x86_64.tar.gz, php-8.1.26-fpm-linux-aarch64.tar.gz, php-8.1.27-micro-linux-aarch64.tar.gz
    ///
    fn file_name(&self) -> String {
        let version = self
            .version
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default();
        match self.category() {
            BuildCategory::WinMin | BuildCategory::WinMax => {
                format!("php-{}-{}-win.zip", version, self.build_type())
            }
            BuildCategory::Bulk | BuildCategory::Common | BuildCategory::Minimal => format!(
                "php-{}-{}-{}-{}.tar.gz",
                version,
                self.build_type(),
                self.os(),
                self.arch()
            ),
        }
    }

    fn arch(&self) -> String {
        self.arch.clone().unwrap_or_else(|| match ARCH {
            "x86_64" | "x86" => "x86_64".to_string(),
            "aarch64" | "arm" => "aarch64".to_string(),
            _ => panic!("Unsupported architecture: {}", ARCH),
        })
    }

    fn build_type(&self) -> String {
        self.build_type.clone().unwrap_or_else(|| "cli".to_string())
    }

    pub fn version_bound(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    fn os(&self) -> String {
        self.os.clone().unwrap_or_else(|| match OS {
            "linux" => "linux".to_string(),
            "macos" => "macos".to_string(),
            "windows" => "win".to_string(),
            _ => panic!("Unsupported operating system: {}", OS),
        })
    }

    fn with_version(&self, version: &Version) -> Self {
        Self {
            category: self.category.clone(),
            version: Some(version.clone()),
            os: self.os.clone(),
            arch: self.arch.clone(),
            build_type: self.build_type.clone(),
        }
    }
}

pub struct Api {
    client: blocking::Client,
    base_url: String,
    options: ApiOptions,
    no_cache: bool,
}

impl Api {
    pub fn new(options: ApiOptions) -> Self {
        Self {
            options,
            client: blocking::Client::new(),
            base_url: "https://dl.static-php.dev/static-php-cli".to_string(),
            no_cache: false,
        }
    }

    pub fn with_no_cache(mut self, no_cache: bool) -> Self {
        self.no_cache = no_cache;
        self
    }

    pub fn fetch_latest_version(&self) -> (Version, bool) {
        let os_needle = self.options.os();
        let arch_needle = self.options.arch();
        let build_type_needle = self.options.build_type();
        let version_bound = self.options.version_bound();

        let (data, from_cache) = self.fetch_versions().unwrap();
        let versions = data
            .into_iter()
            .filter(|resp| {
                let version_match = if let Some(v) = resp.version() {
                    if let Some(bound) = version_bound {
                        v.major == bound.major && v.minor == bound.minor
                    } else {
                        true
                    }
                } else {
                    false
                };

                let name_match = match self.options.category() {
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
            .filter_map(|resp| resp.version());

        let mut highest_version = Version::parse("0.0.0").unwrap();
        for resp_version in versions {
            if highest_version < resp_version {
                highest_version = resp_version.clone();
            }
        }

        (highest_version, from_cache)
    }

    pub fn fetch_versions(&self) -> Result<(Vec<SpcJsonResponse>, bool), reqwest::Error> {
        let category = self.options.category();
        let cache = Cache::new();

        if !self.no_cache && cache.is_valid(&category) {
            if let Some(cached_data) = cache.read(&category) {
                return Ok((cached_data, true));
            }
        }

        let url = self.options.to_url(&self.base_url);
        let response = self.client.get(url).send()?;
        let data: Vec<SpcJsonResponse> = response.json()?;

        if let Err(e) = cache.write(&category, &data) {
            eprintln!("Warning: Failed to write cache: {}", e);
        }

        Ok((data, false))
    }

    pub fn download(&self, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.options.to_download_url(&self.base_url);
        println!("Downloading from: {}", url);

        let mut response = self.client.get(url).send()?;
        let mut file = std::fs::File::create(output_path)?;
        std::io::copy(&mut response, &mut file)?;

        println!("Downloaded to: {}", output_path);
        Ok(())
    }

    pub fn download_url(&self, version: &Version) -> String {
        self.options
            .with_version(version)
            .to_download_url(&self.base_url)
    }
}
