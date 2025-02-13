mod git;
mod file_utils;
mod args;
pub mod parser;

pub use args::Args;
pub use git::repository::{open_repo, find_revision, find_tree};
pub use git::diff::diff_trees;
pub use file_utils::content::{is_likely_binary, file_extension_matches, print_file_content};
pub use parser::{CodeParser, SupportedLanguage};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_file_extension_matches() {
        let path = PathBuf::from("test.rs");
        let extensions = vec!["rs".to_string(), "go".to_string()];
        assert!(file_extension_matches(&path, &extensions));

        let path = PathBuf::from("test.js");
        assert!(!file_extension_matches(&path, &extensions));
    }
}