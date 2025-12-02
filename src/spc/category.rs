use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::env::consts::OS;
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

    pub fn all() -> Vec<BuildCategory> {
        vec![
            BuildCategory::Bulk,
            BuildCategory::Common,
            BuildCategory::Minimal,
            BuildCategory::WinMin,
            BuildCategory::WinMax,
        ]
    }
}
