use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

fn setup_test_repo() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    
    // Create a simple Rust project structure
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::write(
        temp.path().join("src/main.rs"),
        r#"fn main() {
    println!("Hello, world!");
}
"#,
    ).unwrap();
    
    fs::write(
        temp.path().join("src/lib.rs"),
        r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"#,
    ).unwrap();
    
    temp
}

#[test]
fn test_basic_walk() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = setup_test_repo();
    
    let mut cmd = Command::cargo_bin("repo_walker")?;
    cmd.args(["--path", temp_dir.path().to_str().unwrap()]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Repository Snapshot"))
        .stdout(predicate::str::contains("Directory Structure"))
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("Analysis Summary"))
        .stdout(predicate::str::contains("Total tokens processed:"));
    
    Ok(())
}

#[test]
fn test_pattern_matching() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = setup_test_repo();
    
    let mut cmd = Command::cargo_bin("repo_walker")?;
    cmd.args([
        "--path", temp_dir.path().to_str().unwrap(),
        "--pattern", "fn.*add",
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pub fn add"))
        .stdout(predicate::str::contains("Captured:"));
    
    Ok(())
}

#[test]
fn test_extension_filter() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = setup_test_repo();
    
    // Add a non-Rust file
    fs::write(
        temp_dir.path().join("README.md"),
        "# Test Project\n\nThis is a test project.",
    )?;
    
    let mut cmd = Command::cargo_bin("repo_walker")?;
    cmd.args([
        "--path", temp_dir.path().to_str().unwrap(),
        "--extensions", "rs",
    ]);
    
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    
    // Check that we have the expected Rust files
    assert!(stdout.contains("File: ") && stdout.contains("src/main.rs"));
    assert!(stdout.contains("File: ") && stdout.contains("src/lib.rs"));
    
    // Split output into sections
    let sections: Vec<&str> = stdout.split("================================================================").collect();
    
    // Find the file contents sections (they start with "File: ")
    let file_sections: Vec<&str> = sections.iter()
        .filter(|section| section.trim().starts_with("File: "))
        .copied()
        .collect();
    
    // Check that README.md is not in any of the file content sections
    assert!(file_sections.iter().all(|section| !section.contains("README.md")));
    
    Ok(())
}

#[test]
fn test_strip_comments_cli() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.rs");
    
    // Create a test Rust file with comments
    fs::write(&test_file, r#"
// Line comment
fn main() {
    /* Block comment */
    println!("Hello"); // End of line comment
    /* Multi
       line
       comment */
}
"#)?;

    // Run with --strip-comments
    let mut cmd = Command::cargo_bin("repo_walker")?;
    let assert = cmd
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--strip-comments")
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("fn main()"))
        .stdout(predicate::str::contains("println!(\"Hello\");"))
        .stdout(predicate::str::contains("test.rs"))
        .stdout(predicate::str::contains("tokens"))
        .stdout(predicate::str::contains("=".repeat(80)))
        .stdout(predicate::str::contains("Repository Snapshot"))
        .stdout(predicate::str::contains("Analysis Summary"))
        .stdout(predicate::str::contains("Total tokens processed"))
        .stdout(predicate::str::contains("GPT-4 context window sizes"))
        .stdout(predicate::str::contains("8K context"))
        .stdout(predicate::str::contains("32K context"));

    // Run without --strip-comments
    let mut cmd = Command::cargo_bin("repo_walker")?;
    let assert = cmd
        .arg("--path")
        .arg(temp_dir.path())
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("// Line comment"))
        .stdout(predicate::str::contains("/* Block comment */"))
        .stdout(predicate::str::contains("// End of line comment"))
        .stdout(predicate::str::contains("fn main()"))
        .stdout(predicate::str::contains("println!(\"Hello\");"));
    
    Ok(())
} 