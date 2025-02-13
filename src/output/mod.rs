use std::path::Path;
use colored::*;
use tiktoken_rs::p50k_base;
use ignore::WalkBuilder;
use std::collections::BTreeMap;
use regex::Regex;

#[cfg(test)]
mod tests;

pub struct OutputFormatter {
    total_tokens: usize,
    encoding: tiktoken_rs::CoreBPE,
    extensions: Option<Vec<String>>,
    excludes: Option<Vec<Regex>>,
}

impl OutputFormatter {
    pub fn new() -> Self {
        Self {
            total_tokens: 0,
            encoding: p50k_base().unwrap(),
            extensions: None,
            excludes: None,
        }
    }

    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = Some(extensions.into_iter().map(|e| e.to_lowercase()).collect());
        self
    }

    pub fn with_excludes(mut self, excludes: Vec<String>) -> Self {
        self.excludes = Some(excludes.into_iter()
            .filter_map(|pattern| Regex::new(&pattern).ok())
            .collect());
        self
    }

    pub fn print_header(&self, repo_name: &str, commit_sha: &str) {
        println!("{}", "================================================================".blue());
        println!("Repository Snapshot: {} @ {}", repo_name.green(), commit_sha.yellow());
        println!("{}", "================================================================".blue());
    }

    fn should_include_file(&self, path: &Path) -> bool {
        // Check file extension
        if let Some(ref extensions) = self.extensions {
            let extension = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();
            if !extensions.contains(&extension) {
                return false;
            }
        }

        // Check exclude patterns
        if let Some(ref excludes) = self.excludes {
            let path_str = path.to_string_lossy();
            if excludes.iter().any(|re| re.is_match(&path_str)) {
                return false;
            }
        }

        true
    }

    pub fn print_directory_structure(&self, root: &Path) {
        println!("\n{}", "Directory Structure".blue());
        println!("{}", "================================================================".blue());
        
        // Create a map to store directory structure
        let mut dir_map: BTreeMap<String, bool> = BTreeMap::new();
        
        // Use WalkBuilder to respect .gitignore
        let walker = WalkBuilder::new(root)
            .hidden(false)  // Show hidden files unless in .gitignore
            .git_ignore(true)  // Respect .gitignore
            .build();
        
        // First pass: collect all paths
        for entry in walker {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Ok(relative) = path.strip_prefix(root) {
                    if relative.as_os_str().is_empty() {
                        continue;
                    }

                    // Skip files that don't match our criteria
                    if !entry.file_type().map_or(false, |ft| ft.is_dir()) && !self.should_include_file(path) {
                        continue;
                    }

                    let path_str = relative.to_string_lossy().to_string();
                    dir_map.insert(path_str, entry.file_type().map_or(false, |ft| ft.is_dir()));
                }
            }
        }
        
        // Second pass: print the tree
        let mut is_last_at_depth = vec![];
        
        for (path_str, is_dir) in dir_map.iter() {
            let components: Vec<&str> = path_str.split('/').collect();
            let depth = components.len();
            
            // Adjust the is_last_at_depth vector
            while is_last_at_depth.len() < depth {
                is_last_at_depth.push(false);
            }
            is_last_at_depth.truncate(depth);
            
            // Calculate if this is the last item at its depth
            if let Some(next) = dir_map.range::<String, _>((path_str.to_string())..).nth(1) {
                let next_components: Vec<&str> = next.0.split('/').collect();
                is_last_at_depth[depth - 1] = next_components.len() <= depth || 
                    !next.0.starts_with(&format!("{}/", path_str));
            } else {
                is_last_at_depth[depth - 1] = true;
            }
            
            // Print the appropriate prefix
            let mut prefix = String::new();
            for (i, &is_last) in is_last_at_depth[..depth-1].iter().enumerate() {
                if i > 0 {
                    prefix.push_str(if is_last { "    " } else { "│   " });
                }
            }
            prefix.push_str(if is_last_at_depth[depth-1] { "└── " } else { "├── " });
            
            // Print the entry
            let name = components.last().unwrap();
            if *is_dir {
                println!("{}{}/", prefix, name.blue());
            } else {
                println!("{}{}", prefix, name);
            }
        }
    }

    pub fn print_file_contents(&mut self, path: &Path, contents: &str) {
        // Skip files that don't match our criteria
        if !self.should_include_file(path) {
            return;
        }

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
        
        // Print filter information
        if let Some(ref extensions) = self.extensions {
            println!("File extensions: {}", extensions.join(", "));
        }
        if let Some(ref excludes) = self.excludes {
            println!("Exclude patterns: {}", excludes.iter()
                .map(|re| re.as_str().to_string())
                .collect::<Vec<_>>()
                .join(", "));
        }
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