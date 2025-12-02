use chrono::{DateTime, Local, NaiveDateTime, Utc};
use clap::ValueEnum;
use reqwest::blocking;
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    env::consts::{ARCH, OS},
    fs,
    io::{Read, Write},
    path::PathBuf,
};
use strum::{Display, EnumString};

#[derive(Clone, ValueEnum, EnumString, Display, Serialize, Deserialize)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum BuildCategory {
    Bulk,
    Common,
    Minimal,
    WinMin,
    WinMax,
}

impl BuildCategory {
    pub fn default_for_os() -> Self {
        match OS {
            "windows" => BuildCategory::WinMax,
            _ => BuildCategory::Bulk,
        }
    }
}

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
}

pub struct Api {
    client: blocking::Client,
    base_url: String,
    options: ApiOptions,
    no_cache: bool,
}

// Cache implementation
struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("spc-version");

        Self { cache_dir }
    }

    fn cache_file_path(&self, category: &BuildCategory) -> PathBuf {
        self.cache_dir
            .join(format!("{}.json", category.to_string().to_lowercase()))
    }

    fn is_valid(&self, category: &BuildCategory) -> bool {
        let path = self.cache_file_path(category);
        if !path.exists() {
            return false;
        }

        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                let modified_time: DateTime<Local> = modified.into();
                let now = Local::now();

                return modified_time.date_naive() == now.date_naive();
            }
        }

        false
    }

    fn read(&self, category: &BuildCategory) -> Option<Vec<SpcJsonResponse>> {
        let path = self.cache_file_path(category);
        let mut file = fs::File::open(&path).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        serde_json::from_str(&contents).ok()
    }

    fn write(
        &self,
        category: &BuildCategory,
        data: &[SpcJsonResponse],
    ) -> Result<(), std::io::Error> {
        fs::create_dir_all(&self.cache_dir)?;
        let path = self.cache_file_path(category);
        let mut file = fs::File::create(&path)?;
        let json = serde_json::to_string_pretty(data)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
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

    pub fn fetch_latest_version(&self) -> Version {
        let os_needle = self.options.os();
        let arch_needle = self.options.arch();
        let build_type_needle = self.options.build_type();
        let version_bound = self.options.version_bound();

        let versions = self
            .fetch_versions()
            .unwrap()
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

        highest_version
    }

    pub fn fetch_versions(&self) -> Result<Vec<SpcJsonResponse>, reqwest::Error> {
        let category = self.options.category();
        let cache = Cache::new();

        // Check cache first (unless no_cache is set)
        if !self.no_cache && cache.is_valid(&category) {
            if let Some(cached_data) = cache.read(&category) {
                println!("Using cached data for category: {}", category);
                return Ok(cached_data);
            }
        }

        let url = self.options.to_url(&self.base_url);
        println!("Fetching from: {}", url);

        let response = self.client.get(url).send()?;
        let data: Vec<SpcJsonResponse> = response.json()?;

        if let Err(e) = cache.write(&category, &data) {
            eprintln!("Warning: Failed to write cache: {}", e);
        }

        Ok(data)
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpcJsonResponse {
    is_dir: bool,
    full_path: String,
    name: String,
    #[serde(deserialize_with = "deserialize_size")]
    size: String,
    #[serde(deserialize_with = "deserialize_datetime")]
    last_modified: DateTime<chrono::Utc>,
    #[serde(default, deserialize_with = "deserialize_download_count")]
    download_count: u32,
    is_parent: bool,
}

impl SpcJsonResponse {
    pub fn version(&self) -> Option<Version> {
        let mut expected_extensions = [".tar.gz", ".zip"].iter();

        if !expected_extensions.any(|ext| self.name.ends_with(ext)) {
            return None;
        }

        let name_segments = self.name.split("-").collect::<Vec<&str>>();

        let version_str = name_segments
            .get(1)
            .expect(format!("No version string found in name: {}", self.name).as_str());

        Version::parse(version_str).ok()
    }
}

fn deserialize_size<'de, D>(deser: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(u64),
    }

    match StringOrInt::deserialize(deser)? {
        StringOrInt::String(s) => Ok(s),
        StringOrInt::Int(i) => Ok(i.to_string()),
    }
}

fn deserialize_datetime<'de, D>(deser: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deser)?;

    NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc())
        .map_err(serde::de::Error::custom)
}

fn deserialize_download_count<'de, D>(deser: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(u32),
    }

    match StringOrInt::deserialize(deser)? {
        StringOrInt::String(s) => {
            if s.is_empty() {
                Ok(0)
            } else {
                s.parse::<u32>().map_err(serde::de::Error::custom)
            }
        }
        StringOrInt::Int(i) => Ok(i),
    }
}

pub const SPC_OS_OPTIONS: [&str; 3] = ["linux", "windows", "macos"];

pub const SPC_ARCH_OPTIONS: [&str; 2] = ["x86_64", "aarch64"];

pub const SPC_PHP_BUILD_TYPE_OPTIONS: [&str; 3] = ["micro", "fpm", "cli"];

pub const SPC_MINIMAL_PHP_EXTENSIONS: [&str; 8] = [
    "iconv",
    "pcntl",
    "posix",
    "mbstring",
    "filter",
    "tokenizer",
    "zlib",
    "phar",
];

pub const SPC_MINIMAL_PHP_LIBRARIES: [&str; 6] =
    ["lib-base", "libiconv", "micro", "frankenphp", "php", "zlib"];

pub const SPC_COMMON_PHP_EXTENSIONS: [&str; 38] = [
    "bcmath",
    "bz2",
    "calendar",
    "ctype",
    "curl",
    "dom",
    "exif",
    "fileinfo",
    "filter",
    "ftp",
    "zlib",
    "gd",
    "gmp",
    "iconv",
    "xml",
    "mbstring",
    "mbregex",
    "mysqlnd",
    "openssl",
    "pcntl",
    "pdo",
    "pdo_mysql",
    "sqlite3",
    "pdo_sqlite",
    "pgsql",
    "pdo_pgsql",
    "phar",
    "posix",
    "session",
    "redis",
    "simplexml",
    "libxml",
    "soap",
    "sockets",
    "tokenizer",
    "xmlwriter",
    "xmlreader",
    "zip",
];

pub const SPC_COMMON_PHP_LIBRARIES: [&str; 42] = [
    "lib-base",
    "micro",
    "frankenphp",
    "attr",
    "libacl",
    "brotli",
    "watcher",
    "php",
    "bzip2",
    "zlib",
    "openssl",
    "libssh2",
    "libiconv",
    "xz",
    "libxml2",
    "nghttp3",
    "ngtcp2",
    "nghttp2",
    "zstd",
    "libcares",
    "gmp",
    "libsodium",
    "ldap",
    "ncurses",
    "gettext",
    "libunistring",
    "idn2",
    "libedit",
    "krb5",
    "curl",
    "libpng",
    "libavif",
    "libwebp",
    "libjpeg",
    "freetype",
    "onig",
    "sqlite",
    "icu",
    "libxslt",
    "postgresql",
    "liblz4",
    "libzip",
];

pub const SPC_BULK_PHP_EXTENSIONS: [&str; 57] = [
    "apcu",
    "bcmath",
    "bz2",
    "calendar",
    "ctype",
    "curl",
    "dba",
    "dom",
    "zlib",
    "openssl",
    "sockets",
    "event",
    "exif",
    "fileinfo",
    "filter",
    "ftp",
    "gd",
    "gmp",
    "iconv",
    "imagick",
    "imap",
    "intl",
    "mbstring",
    "mbregex",
    "mysqlnd",
    "mysqli",
    "opcache",
    "opentelemetry",
    "pcntl",
    "pdo",
    "pdo_mysql",
    "pgsql",
    "phar",
    "posix",
    "protobuf",
    "readline",
    "session",
    "redis",
    "shmop",
    "simplexml",
    "xml",
    "libxml",
    "soap",
    "sodium",
    "sqlite3",
    "swoole-hook-pgsql",
    "swoole-hook-mysql",
    "swoole-hook-sqlite",
    "swoole",
    "sysvmsg",
    "sysvsem",
    "sysvshm",
    "tokenizer",
    "xmlreader",
    "xmlwriter",
    "xsl",
    "zip",
];

pub const SPC_BULK_PHP_LIBRARIES: [&str; 54] = [
    "lib-base",
    "micro",
    "frankenphp",
    "attr",
    "libacl",
    "brotli",
    "watcher",
    "php",
    "bzip2",
    "zlib",
    "openssl",
    "libssh2",
    "libiconv",
    "xz",
    "libxml2",
    "nghttp3",
    "ngtcp2",
    "nghttp2",
    "zstd",
    "libcares",
    "gmp",
    "libsodium",
    "ldap",
    "ncurses",
    "gettext",
    "libunistring",
    "idn2",
    "libedit",
    "krb5",
    "curl",
    "qdbm",
    "libevent",
    "libpng",
    "libavif",
    "libwebp",
    "libjpeg",
    "freetype",
    "libjxl",
    "lerc",
    "jbig",
    "libtiff",
    "libde265",
    "libaom",
    "libheif",
    "libzip",
    "imagemagick",
    "imap",
    "icu",
    "onig",
    "libxslt",
    "postgresql",
    "liblz4",
    "sqlite",
    "liburing",
];

pub const SPC_WINDOWS_MIN_EXTENSIONS: [&str; 7] = [
    "ctype",
    "fileinfo",
    "filter",
    "iconv",
    "mbstring",
    "tokenizer",
    "phar",
];

pub const SPC_WINDOWS_MAX_EXTENSIONS: [&str; 50] = [
    "amqp",
    "apcu",
    "bcmath",
    "bz2",
    "calendar",
    "ctype",
    "curl",
    "dba",
    "dom",
    "ds",
    "exif",
    "ffi",
    "fileinfo",
    "filter",
    "ftp",
    "gd",
    "iconv",
    "igbinary",
    "libxml",
    "mbregex",
    "mbstring",
    "mysqli",
    "mysqlnd",
    "opcache",
    "openssl",
    "pdo",
    "pdo_mysql",
    "pdo_sqlite",
    "pdo_sqlsrv",
    "phar",
    "rar",
    "redis",
    "session",
    "shmop",
    "simdjson",
    "simplexml",
    "soap",
    "sockets",
    "sqlite3",
    "sqlsrv",
    "ssh2",
    "sysvshm",
    "tokenizer",
    "xml",
    "xmlreader",
    "xmlwriter",
    "yac",
    "yaml",
    "zip",
    "zlib",
];
