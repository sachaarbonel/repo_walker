use std::path::Path;
use colored::*;
use tiktoken_rs::p50k_base;

#[cfg(test)]
mod tests;

pub struct OutputFormatter {
    total_tokens: usize,
    encoding: tiktoken_rs::CoreBPE,
}

impl OutputFormatter {
    pub fn new() -> Self {
        Self {
            total_tokens: 0,
            encoding: p50k_base().unwrap(),
        }
    }

    pub fn print_header(&self, repo_name: &str, commit_sha: &str) {
        println!("{}", "================================================================".blue());
        println!("Repository Snapshot: {} @ {}", repo_name.green(), commit_sha.yellow());
        println!("{}", "================================================================".blue());
    }

    pub fn print_directory_structure(&self, root: &Path) {
        println!("\n{}", "Directory Structure".blue());
        println!("{}", "================================================================".blue());
        self.print_dir_recursive(root, 0);
    }

    fn print_dir_recursive(&self, dir: &Path, depth: usize) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap().to_string_lossy();
                
                // Skip .git directory and other hidden files
                if name.starts_with('.') {
                    continue;
                }

                let prefix = "  ".repeat(depth);
                if path.is_dir() {
                    println!("{}├── {}/", prefix, name.blue());
                    self.print_dir_recursive(&path, depth + 1);
                } else {
                    println!("{}├── {}", prefix, name);
                }
            }
        }
    }

    pub fn print_file_contents(&mut self, path: &Path, contents: &str) {
        let tokens = self.count_tokens(contents);
        self.total_tokens += tokens;

        println!("\n{}", "=".repeat(80).blue());
        println!("File: {} (≈{} tokens)", path.display().to_string().green(), tokens);
        println!("{}", "=".repeat(80).blue());

        // Print file contents with line numbers
        for (i, line) in contents.lines().enumerate() {
            println!("{:4}│ {}", i + 1, line);
        }
    }

    pub fn print_summary(&self) {
        println!("\n{}", "Analysis Summary".blue());
        println!("{}", "================================================================".blue());
        println!("Total tokens processed: {}", self.total_tokens);
        println!("GPT-4 context window sizes for reference:");
        println!("- 8K context: {}", self.format_token_usage(8192));
        println!("- 32K context: {}", self.format_token_usage(32768));
    }

    fn format_token_usage(&self, context_size: usize) -> String {
        let percentage = (self.total_tokens as f64 / context_size as f64 * 100.0).round();
        format!("{:.1}% used ({}/{})", percentage, self.total_tokens, context_size)
    }

    fn count_tokens(&self, text: &str) -> usize {
        self.encoding.encode_with_special_tokens(text).len()
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new()
    }
} 