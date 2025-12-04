use chrono::{DateTime, NaiveDateTime, Utc};
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SpcJsonResponse {
    is_dir: bool,
    full_path: String,
    pub name: String,
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
        let expected_extensions = [".tar.gz", ".zip"];

        if !expected_extensions
            .iter()
            .any(|ext| self.name.ends_with(ext))
        {
            return None;
        }

        let version_str = self.name.split('-').nth(1)?;

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

    if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
        return Ok(dt.with_timezone(&Utc));
    }

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
