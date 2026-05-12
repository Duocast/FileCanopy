use assert_cmd::Command;
use predicates::str;

#[test]
fn prints_help() {
    Command::cargo_bin("filecanopy")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(str::contains("filecanopy"));
}

#[test]
fn prints_version() {
    Command::cargo_bin("filecanopy")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}
