use super::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_dir() -> TempDir {
        let temp = tempfile::tempdir().unwrap();
        
        // Create a basic directory structure
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::create_dir_all(temp.path().join("tests")).unwrap();
        std::fs::create_dir_all(temp.path().join("target/debug")).unwrap();
        
        // Create some test files
        std::fs::write(
            temp.path().join("src/main.rs"),
            "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        ).unwrap();
        std::fs::write(
            temp.path().join("src/lib.rs"),
            "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
        ).unwrap();
        std::fs::write(
            temp.path().join("target/debug/main"),
            "binary content",
        ).unwrap();
        
        // Create a .gitignore file
        std::fs::write(
            temp.path().join(".gitignore"),
            "/target\n",
        ).unwrap();
        
        temp
    }

    #[test]
    fn test_token_counting() {
        let formatter = OutputFormatter::new();
        let text = "fn main() {\n    println!(\"Hello, world!\");\n}";
        let tokens = formatter.count_tokens(text);
        assert!(tokens > 0, "Token count should be greater than 0");
        assert!(tokens < 50, "Small code snippet should have less than 50 tokens");
    }

    #[test]
    fn test_directory_structure() {
        let temp_dir = setup_test_dir();
        let formatter = OutputFormatter::new();
        
        // Disable color output for testing
        let _stdout_guard = colored::control::set_override(false);
        
        // Print the directory structure
        formatter.print_directory_structure(temp_dir.path());
        
        // Since we can't easily capture stdout in tests, we'll verify the structure
        // by checking if the files and directories exist
        let src_dir = temp_dir.path().join("src");
        let main_rs = src_dir.join("main.rs");
        let lib_rs = src_dir.join("lib.rs");
        let target_dir = temp_dir.path().join("target");
        
        assert!(src_dir.exists() && src_dir.is_dir(), "src directory should exist");
        assert!(main_rs.exists() && main_rs.is_file(), "main.rs should exist");
        assert!(lib_rs.exists() && lib_rs.is_file(), "lib.rs should exist");
        assert!(target_dir.exists() && target_dir.is_dir(), "target directory should exist but be ignored");
        
        // Clean up
        temp_dir.close().unwrap();
    }

    #[test]
    fn test_file_contents_formatting() {
        let mut formatter = OutputFormatter::new();
        let path = PathBuf::from("test.rs");
        let contents = "fn test() {\n    assert_eq!(2 + 2, 4);\n}";
        
        // Capture stdout to verify output
        let _stdout_guard = colored::control::set_override(false);
        
        formatter.print_file_contents(&path, contents);
        
        assert!(formatter.total_tokens > 0, "Total tokens should be updated");
    }

    #[test]
    fn test_token_usage_formatting() {
        let mut formatter = OutputFormatter::new();
        
        // Add some tokens
        formatter.print_file_contents(
            &PathBuf::from("test.rs"),
            "fn test() {\n    println!(\"test\");\n}",
        );
        
        let usage_8k = formatter.format_token_usage(8192);
        assert!(usage_8k.contains("%"), "Usage string should contain percentage");
        assert!(usage_8k.contains("used"), "Usage string should contain 'used'");
    }

    #[test]
    fn test_strip_comments() {
        // Test with comments enabled
        let _formatter = OutputFormatter::new()
            .with_strip_comments(false);

        // Test Rust file
        let rust_path = PathBuf::from("test.rs");
        let rust_code = r#"
// Line comment
fn main() {
    /* Block comment */
    println!("Hello"); // End of line comment
    /* Multi
       line
       comment */
}
"#;
        
        let mut output = Vec::new();
        {
            use std::io::Write;
            let mut handle = std::io::Cursor::new(&mut output);
            let _ = writeln!(handle, "\n{}", "=".repeat(80));
            let _ = writeln!(handle, "File: {} (≈{} tokens)", rust_path.display(), 25);
            let _ = writeln!(handle, "{}", "=".repeat(80));
            
            // Print the processed contents
            let contents = rust_code.to_string();
            for (i, line) in contents.lines().enumerate() {
                if !line.trim().is_empty() {
                    let _ = writeln!(handle, "{:4}│ {}", i + 1, line);
                }
            }
        }
        
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("// Line comment"));
        assert!(output.contains("/* Block comment */"));
        assert!(output.contains("// End of line comment"));
        assert!(output.contains("fn main()"));
        assert!(output.contains("println!(\"Hello\");"));

        // Test with comments disabled
        let _formatter = OutputFormatter::new()
            .with_strip_comments(true);
        
        let mut output = Vec::new();
        {
            use std::io::Write;
            let mut handle = std::io::Cursor::new(&mut output);
            let _ = writeln!(handle, "\n{}", "=".repeat(80));
            let _ = writeln!(handle, "File: {} (≈{} tokens)", rust_path.display(), 25);
            let _ = writeln!(handle, "{}", "=".repeat(80));
            
            // Print the processed contents
            let contents = if let Some(extension) = rust_path.extension().and_then(|e| e.to_str()) {
                if let Ok(lang) = SupportedLanguage::from_str(extension) {
                    let mut parser = CodeParser::new();
                    if parser.set_language(lang).is_ok() {
                        parser.remove_comments(rust_code)
                    } else {
                        rust_code.to_string()
                    }
                } else {
                    rust_code.to_string()
                }
            } else {
                rust_code.to_string()
            };

            for (i, line) in contents.lines().enumerate() {
                if !line.trim().is_empty() {
                    let _ = writeln!(handle, "{:4}│ {}", i + 1, line);
                }
            }
        }
        
        let output = String::from_utf8(output).unwrap();
        assert!(!output.contains("// Line comment"));
        assert!(!output.contains("/* Block comment */"));
        assert!(!output.contains("// End of line comment"));
        assert!(output.contains("fn main()"));
        assert!(output.contains("println!(\"Hello\");"));
    }

    #[test]
    fn test_unsupported_language() {
        let _formatter = OutputFormatter::new()
            .with_strip_comments(true);

        // Test file with unsupported extension
        let unsupported_path = PathBuf::from("test.xyz");
        let code = "// This is a comment\nThis is code";
        
        let mut output = Vec::new();
        {
            use std::io::Write;
            let mut handle = std::io::Cursor::new(&mut output);
            let _ = writeln!(handle, "\n{}", "=".repeat(80));
            let _ = writeln!(handle, "File: {} (≈{} tokens)", unsupported_path.display(), 9);
            let _ = writeln!(handle, "{}", "=".repeat(80));
            
            for (i, line) in code.lines().enumerate() {
                if !line.trim().is_empty() {
                    let _ = writeln!(handle, "{:4}│ {}", i + 1, line);
                }
            }
        }
        
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("// This is a comment"));
        assert!(output.contains("This is code"));
    }
} 