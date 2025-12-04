use assert_cmd::{Command, cargo::cargo_bin_cmd};
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn cmd() -> Command {
    cargo_bin_cmd!("spc-utils")
}

#[test]
fn latest_retuns_valid_version() {
    cmd()
        .args(["latest", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"Latest Version: \d+\.\d+\.\d+").unwrap());
}

#[test]
fn latest_with_version_filter() {
    cmd()
        .args(["check-update", "-V", "8.0.0", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"Update available: 8.0.0 -> 8.0.\d+").unwrap());
}

#[test]
fn latest_with_category() {
    cmd()
        .args(["latest", "-C", "common", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"Latest Version: \d+\.\d+\.\d+").unwrap());
}

#[test]
fn latest_with_os_and_arch() {
    cmd()
        .args(["latest", "-O", "linux", "-A", "x86_64", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"Latest Version: \d+\.\d+\.\d+").unwrap());
}

#[test]
fn latest_with_build_type() {
    cmd()
        .args(["latest", "-B", "micro", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"Latest Version: \d+\.\d+\.\d+").unwrap());
}

#[test]
fn check_update_detects_outdated_version() {
    cmd()
        .args(["check-update", "-V", "8.0.0", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Update available"));
}

#[test]
fn check_update_with_category() {
    cmd()
        .args(["check-update", "-C", "common", "-V", "8.0.0", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Update available"));
}

#[test]
fn check_update_shows_download_url() {
    cmd()
        .args(["check-update", "-V", "8.0.0", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://dl.static-php.dev/"));
}

#[test]
fn download_creates_file() {
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("php-test-binary");

    cmd()
        .args([
            "download",
            "-V",
            "8.0.30",
            "-O",
            "linux",
            "-A",
            "x86_64",
            "-o",
            output_path.to_str().unwrap(),
            "--no-cache",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Downloading from:"))
        .stdout(predicate::str::contains("Downloaded to:"));

    assert!(output_path.exists());
    let metadata = fs::metadata(&output_path).unwrap();
    assert!(metadata.len() > 0);
}

#[test]
fn download_with_category_and_build_type() {
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("php-micro");

    cmd()
        .args([
            "download",
            "-C",
            "minimal",
            "-V",
            "8.4",
            "-B",
            "micro",
            "-o",
            output_path.to_str().unwrap(),
            "--no-cache",
        ])
        .assert()
        .success();

    assert!(output_path.exists());
}

#[test]
fn download_requires_output_flag() {
    cmd()
        .args(["download", "-V", "8.0.0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--output"));
}

#[test]
fn cache_path_returns_directory() {
    cmd()
        .args(["cache", "path"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r".+").unwrap());
}

#[test]
fn cache_list_succeeds() {
    cmd().args(["cache", "list"]).assert().success();
}

#[test]
fn cache_clear_succeeds() {
    let result = cmd().args(["cache", "clear"]).assert().success();

    let output = result.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("Removed")
            || stdout.contains("No cache files to remove")
            || stderr.contains("Failed to clear cache")
    );
}

#[test]
fn cache_clear_with_category() {
    let result = cmd().args(["cache", "clear", "-C", "bulk"]).assert();

    let output = result.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("Removed")
            || stdout.contains("No cache files to remove")
            || stderr.contains("Failed to clear cache")
    );
}

#[test]
fn invalid_category_fails() {
    cmd().args(["latest", "-C", "foobar"]).assert().failure();
}

#[test]
fn invalid_os_fails() {
    cmd().args(["latest", "-O", "bsd"]).assert().failure();
}

#[test]
fn invalid_arch_fails() {
    cmd().args(["latest", "-A", "arm32"]).assert().failure();
}

#[test]
fn invalid_build_type_fails() {
    cmd().args(["latest", "-B", "debug"]).assert().failure();
}

#[test]
fn version_parsing_with_two_parts() {
    cmd()
        .args(["check-update", "-V", "8.0", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"8\.0\.\d+").unwrap());
}

#[test]
fn version_parsing_with_zero_minor() {
    cmd()
        .args(["latest", "-V", "8.0", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::contains("8.0."));
}

#[test]
fn version_below_8_fails() {
    cmd()
        .args(["latest", "-V", "7.4"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not supported"));
}

#[test]
fn version_7_fails() {
    cmd()
        .args(["check-update", "-V", "7"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("SPC only provides PHP 8.0.0"));
}

#[test]
fn cache_clears_on_different_spc_utils_version() {
    let cache_path_output = cmd()
        .args(["cache", "path"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let cache_dir = PathBuf::from(String::from_utf8_lossy(&cache_path_output).trim());

    fs::create_dir_all(&cache_dir).unwrap();

    let version_file = cache_dir.join(".version");
    fs::write(&version_file, "0.0.0-old").unwrap();

    let dummy_cache = cache_dir.join("bulk.json");
    fs::write(&dummy_cache, r#"[{"test": "data"}]"#).unwrap();

    assert!(version_file.exists());
    assert!(dummy_cache.exists());

    cmd().args(["cache", "path"]).assert().success();

    assert!(version_file.exists());
    let new_version = fs::read_to_string(&version_file).unwrap();
    assert_ne!(new_version.trim(), "0.0.0-old");
    assert!(!dummy_cache.exists());
}
