use chrono::{DateTime, Local, NaiveTime};
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};

use super::{BuildCategory, SpcJsonResponse};

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct CacheFileInfo {
    pub category: BuildCategory,
    pub size: u64,
    pub modified: DateTime<Local>,
    pub expires: DateTime<Local>,
    pub entry_count: usize,
}

pub struct Cache {
    cache_dir: PathBuf,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("spc-utils");

        let cache = Self { cache_dir };
        cache.check_version();
        cache
    }

    fn version_file_path(&self) -> PathBuf {
        self.cache_dir.join(".version")
    }

    fn check_version(&self) {
        let version_file = self.version_file_path();

        if let Ok(mut file) = fs::File::open(&version_file) {
            let mut stored_version = String::new();
            if file.read_to_string(&mut stored_version).is_ok()
                && stored_version.trim() == CRATE_VERSION
            {
                return;
            }
        }

        let _ = self.clear(None);
        self.write_version();
    }

    fn write_version(&self) {
        if fs::create_dir_all(&self.cache_dir).is_ok()
            && let Ok(mut file) = fs::File::create(self.version_file_path())
        {
            let _ = file.write_all(CRATE_VERSION.as_bytes());
        }
    }

    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    pub fn cache_file_path(&self, category: &BuildCategory) -> PathBuf {
        self.cache_dir
            .join(format!("{}.json", category.to_string().to_lowercase()))
    }

    pub fn is_valid(&self, category: &BuildCategory) -> bool {
        let path = self.cache_file_path(category);
        if !path.exists() {
            return false;
        }

        if let Ok(metadata) = fs::metadata(&path)
            && let Ok(modified) = metadata.modified()
        {
            let modified_time: DateTime<Local> = modified.into();
            let now = Local::now();
            return modified_time.date_naive() == now.date_naive();
        }

        false
    }

    pub fn read(&self, category: &BuildCategory) -> Option<Vec<SpcJsonResponse>> {
        let path = self.cache_file_path(category);
        let mut file = fs::File::open(&path).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        serde_json::from_str(&contents).ok()
    }

    pub fn write(
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

    pub fn list_cached_files(&self) -> Vec<CacheFileInfo> {
        let mut files = Vec::new();

        for category in BuildCategory::all() {
            let path = self.cache_file_path(&category);
            if path.exists()
                && let Ok(metadata) = fs::metadata(&path)
            {
                let modified: DateTime<Local> = metadata
                    .modified()
                    .map(|t| t.into())
                    .unwrap_or_else(|_| Local::now());

                let expires = modified
                    .date_naive()
                    .succ_opt()
                    .unwrap()
                    .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                    .and_local_timezone(Local)
                    .unwrap();

                let entry_count = self.read(&category).map(|v| v.len()).unwrap_or(0);

                files.push(CacheFileInfo {
                    category,
                    size: metadata.len(),
                    modified,
                    expires,
                    entry_count,
                });
            }
        }

        files
    }

    pub fn clear(&self, category: Option<&BuildCategory>) -> Result<usize, std::io::Error> {
        let mut removed = 0;

        match category {
            Some(cat) => {
                let path = self.cache_file_path(cat);
                if path.exists() {
                    fs::remove_file(&path)?;
                    removed = 1;
                }
            }
            None => {
                for cat in BuildCategory::all() {
                    let path = self.cache_file_path(&cat);
                    if path.exists() {
                        fs::remove_file(&path)?;
                        removed += 1;
                    }
                }
            }
        }

        Ok(removed)
    }
}
