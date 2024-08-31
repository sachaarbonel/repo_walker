use repo_walker::{open_repo, find_revision, find_tree, diff_trees};
use std::path::PathBuf;

#[test]
fn test_open_repo() {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test_data/sample_repo");
    let result = open_repo(&repo_path);
    assert!(result.is_ok());
}

#[test]
fn test_find_revision() {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test_data/sample_repo");
    let repo = open_repo(&repo_path).unwrap();
    
    let result = find_revision(&repo, "HEAD");
    assert!(result.is_ok());
}

#[test]
fn test_diff_trees() {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test_data/sample_repo");
    let repo = open_repo(&repo_path).unwrap();
    
    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    
    let obj1 = find_revision(&repo, "HEAD~1").unwrap();
    let obj2 = find_revision(&repo, "HEAD").unwrap();
    
    let tree1 = find_tree(&repo, obj1, &mut buf1).unwrap();
    let tree2 = find_tree(&repo, obj2, &mut buf2).unwrap();
    
    let result = diff_trees(&repo, tree1, tree2);
    assert!(result.is_ok());
}