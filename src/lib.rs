pub mod args;
pub mod git;
pub mod file_utils;

// Re-export commonly used items
pub use args::Args;
pub use git::repository::{open_repo, find_revision, find_tree};
pub use git::diff::diff_trees;
pub use file_utils::content::{is_likely_binary, file_extension_matches, print_file_content};