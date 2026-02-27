use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

fn write_executable(path: &Path, content: &str) {
    fs::write(path, content).expect("write script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("chmod");
    }
}

fn setup_fake_tooling(include_aikido: bool) -> (TempDir, String) {
    let tmp = TempDir::new().expect("tempdir");
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("bin dir");

    write_executable(
        &bin_dir.join("aiken"),
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "aiken 1.1.21"
  exit 0
fi
if [ "$1" = "build" ] || [ "$1" = "check" ]; then
  exit 0
fi
exit 0
"#,
    );

    if include_aikido {
        write_executable(
            &bin_dir.join("aikido"),
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "aikido 1.0.0"
  exit 0
fi
case "$AIKIDO_MODE" in
  invalid_json)
    echo "not-json"
    exit 0
    ;;
  empty_fail)
    exit 3
    ;;
  critical_findings)
    echo '{"findings":[{"detector":"x","severity":"critical","description":"boom"}],"total":1}'
    exit 2
    ;;
  *)
    echo '{"findings":[],"total":0}'
    exit 0
    ;;
esac
"#,
        );
    }

    let path = bin_dir.display().to_string();
    (tmp, path)
}

fn setup_project() -> TempDir {
    let tmp = TempDir::new().expect("project tempdir");
    fs::write(tmp.path().join("aiken.toml"), "name = \"test/project\"\n")
        .expect("write aiken.toml");
    tmp
}

fn kaido_bin() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("kaido"))
}

#[test]
fn verify_fails_on_invalid_aikido_json() {
    let (_tools, path_env) = setup_fake_tooling(true);
    let project = setup_project();

    let mut cmd = kaido_bin();
    cmd.args(["verify", project.path().to_str().expect("project path")])
        .env("PATH", &path_env)
        .env("AIKIDO_MODE", "invalid_json");

    cmd.assert().failure().stderr(predicates::str::contains(
        "Failed to parse aikido JSON output",
    ));
}

#[test]
fn verify_accepts_non_zero_aikido_exit_with_findings_json() {
    let (_tools, path_env) = setup_fake_tooling(true);
    let project = setup_project();

    let mut cmd = kaido_bin();
    cmd.args(["verify", project.path().to_str().expect("project path")])
        .env("PATH", &path_env)
        .env("AIKIDO_MODE", "critical_findings");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("1 finding(s) (1 high/critical)"));
}

#[test]
fn generate_requires_aikido_when_verification_enabled() {
    let (_tools, path_env) = setup_fake_tooling(false);
    let output = TempDir::new().expect("output tempdir");
    let output_dir: PathBuf = output.path().join("generated");

    let mut cmd = kaido_bin();
    cmd.args([
        "generate",
        "--template",
        "mint",
        "--namespace",
        "myorg",
        "--project-name",
        "my_token",
        "--output",
        output_dir.to_str().expect("output path"),
    ])
    .env("PATH", &path_env);

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("aikido not found on PATH"));
}
