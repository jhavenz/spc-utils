use assert_cmd::{Command, cargo::cargo_bin_cmd};
use predicates::prelude::*;

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
        .args(["check-update", "-v", "8.0.0", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"Update available: 8.0.0 → 8.0.\d+").unwrap());
}

#[test]
fn check_update_detects_outdated_version() {
    cmd()
        .args(["check-update", "-v", "8.0.0", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Update available"));
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
fn invalid_category_fails() {
    cmd().args(["latest", "-C", "foobar"]).assert().failure();
}

#[test]
fn download_url_form_in_check_update() {
    cmd()
        .args(["check-update", "-v", "8.0.0", "--no-cache"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://dl.static-php.dev/"));
}
