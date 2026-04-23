//! Integration tests for `ulf tools skill` CLI commands.

use std::process::Command;
use std::{fs, path::Path};
use tempfile::TempDir;

fn ulf_skill(temp_path: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ulf"))
        .arg("tools")
        .arg("skill")
        .args(args)
        .arg("--root")
        .arg(temp_path)
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute ulf tools skill command")
}

fn ulf_skill_no_root(current_path: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ulf"))
        .arg("tools")
        .arg("skill")
        .args(args)
        .current_dir(current_path)
        .output()
        .expect("Failed to execute ulf tools skill command")
}

fn write_skill(root: &Path, name: &str, contents: &str) {
    let skill_dir = root.join(".claude").join("skills").join(name);
    fs::create_dir_all(&skill_dir).expect("create skill dir");
    fs::write(skill_dir.join("SKILL.md"), contents).expect("write skill file");
}

fn ulf_skill_ok(temp_path: &std::path::Path, args: &[&str]) -> String {
    let output = ulf_skill(temp_path, args);
    assert!(
        output.status.success(),
        "Command 'ulf tools skill {}' failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn ulf_skill_no_root_ok(current_path: &std::path::Path, args: &[&str]) -> String {
    let output = ulf_skill_no_root(current_path, args);
    assert!(
        output.status.success(),
        "Command 'ulf tools skill {}' failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_skill_load_builtin() {
    let temp_dir = TempDir::new().expect("temp dir");
    let temp_path = temp_dir.path();

    let stdout = ulf_skill_ok(temp_path, &["load", "ulf-tools"]);
    assert!(stdout.contains("Ulf Tools"));
    assert!(stdout.contains("ulf tools skill"));
}

#[test]
fn test_skill_load_missing_exits_nonzero() {
    let temp_dir = TempDir::new().expect("temp dir");
    let temp_path = temp_dir.path();

    let output = ulf_skill(temp_path, &["load", "missing-skill"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found"));
}

#[test]
fn test_skill_list_includes_builtins() {
    let temp_dir = TempDir::new().expect("temp dir");
    let temp_path = temp_dir.path();

    let stdout = ulf_skill_ok(temp_path, &["list", "--format", "quiet"]);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.contains(&"ulf-tools"));
    assert!(lines.contains(&"robot-interaction"));
}

#[test]
fn test_skill_list_and_load_user_skill_from_default_dir() {
    let temp_dir = TempDir::new().expect("temp dir");
    let temp_path = temp_dir.path();

    write_skill(
        temp_path,
        "test-driven-development",
        r"---
name: test-driven-development
description: Test generation skill
---

# Test Generation

Loaded from default skills dir.
",
    );

    let list_stdout = ulf_skill_ok(temp_path, &["list", "--format", "quiet"]);
    let list_lines: Vec<&str> = list_stdout.lines().collect();
    assert!(list_lines.contains(&"test-driven-development"));

    let load_stdout = ulf_skill_ok(temp_path, &["load", "test-driven-development"]);
    assert!(load_stdout.contains("<test-driven-development-skill>"));
    assert!(load_stdout.contains("Loaded from default skills dir."));
}

#[test]
fn test_skill_load_finds_nested_skills_dir_when_root_missing() {
    let temp_dir = TempDir::new().expect("temp dir");
    let temp_path = temp_dir.path();

    fs::write(temp_path.join("ulf.yml"), "skills:\n  enabled: true\n").expect("write ulf.yml");

    let repo_dir = temp_path.join("repo");
    let nested_dir = repo_dir.join("nested");
    fs::create_dir_all(&nested_dir).expect("create nested dir");

    write_skill(
        &repo_dir,
        "test-driven-development",
        r"---
name: test-driven-development
description: Test generation skill
---

# Test Generation

Loaded from nested skills dir.
",
    );

    let list_stdout = ulf_skill_no_root_ok(&nested_dir, &["list", "--format", "quiet"]);
    let list_lines: Vec<&str> = list_stdout.lines().collect();
    assert!(list_lines.contains(&"test-driven-development"));

    let load_stdout = ulf_skill_no_root_ok(&nested_dir, &["load", "test-driven-development"]);
    assert!(load_stdout.contains("Loaded from nested skills dir."));
}

#[test]
fn test_skill_load_finds_parent_skills_dir_when_root_nested() {
    let temp_dir = TempDir::new().expect("temp dir");
    let temp_path = temp_dir.path();

    let repo_dir = temp_path.join("repo");
    let workspace_dir = repo_dir.join("ulf-orchestrator");
    let nested_dir = workspace_dir.join("nested");
    fs::create_dir_all(&nested_dir).expect("create nested dir");

    fs::write(
        workspace_dir.join("ulf.yml"),
        "skills:\n  enabled: true\n",
    )
    .expect("write ulf.yml");

    write_skill(
        &repo_dir,
        "test-driven-development",
        r"---
name: test-driven-development
description: Test generation skill
---

# Test Generation

Loaded from parent skills dir.
",
    );

    let list_stdout = ulf_skill_no_root_ok(&nested_dir, &["list", "--format", "quiet"]);
    let list_lines: Vec<&str> = list_stdout.lines().collect();
    assert!(list_lines.contains(&"test-driven-development"));

    let load_stdout = ulf_skill_no_root_ok(&nested_dir, &["load", "test-driven-development"]);
    assert!(load_stdout.contains("Loaded from parent skills dir."));
}

#[test]
fn test_skill_load_finds_parent_skills_dir_when_configured_root_missing() {
    let temp_dir = TempDir::new().expect("temp dir");
    let temp_path = temp_dir.path();

    let repo_dir = temp_path.join("repo");
    let workspace_dir = repo_dir.join("ulf-orchestrator");
    let nested_dir = workspace_dir.join("nested");
    fs::create_dir_all(&nested_dir).expect("create nested dir");

    fs::write(
        workspace_dir.join("ulf.yml"),
        "skills:\n  enabled: true\n  dirs:\n    - .claude/skills\n",
    )
    .expect("write ulf.yml");

    write_skill(
        &repo_dir,
        "test-driven-development",
        r"---
name: test-driven-development
description: Test generation skill
---

# Test Generation

Loaded from configured parent skills dir.
",
    );

    let list_stdout = ulf_skill_no_root_ok(&nested_dir, &["list", "--format", "quiet"]);
    let list_lines: Vec<&str> = list_stdout.lines().collect();
    assert!(list_lines.contains(&"test-driven-development"));

    let load_stdout = ulf_skill_no_root_ok(&nested_dir, &["load", "test-driven-development"]);
    assert!(load_stdout.contains("Loaded from configured parent skills dir."));
}
