use clap::Parser;
use gix::bstr::BString;
use gix::bstr::ByteSlice;
use gix::diff::tree::recorder::Change;
use gix::objs::tree::EntryMode;
use gix::Repository;
use ignore::WalkBuilder;
use regex::Regex;
use repo_walker::diff_trees;
use repo_walker::file_extension_matches;
use repo_walker::find_revision;
use repo_walker::find_tree;
use repo_walker::is_likely_binary;
use repo_walker::open_repo;
use repo_walker::print_file_content;
use repo_walker::Args;
use std::fs;
use std::path::{Path, PathBuf};

struct GitPath(PathBuf);

impl From<&BString> for GitPath {
    fn from(bstring: &BString) -> Self {
        GitPath(PathBuf::from(bstring.to_path_lossy()))
    }
}

impl AsRef<Path> for GitPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.git_from.is_some() || args.git_to.is_some() {
        return print_git_diff(&args);
    }

    let pattern = args.pattern.map(|p| Regex::new(&p)).transpose()?;
    let extensions: Option<Vec<String>> = args
        .extensions
        .map(|exts| exts.into_iter().map(|e| e.to_lowercase()).collect());

    let excludes: Option<Vec<Regex>> = args
        .excludes
        .as_ref()
        .map(|patterns| patterns.iter().map(|p| Regex::new(p).unwrap()).collect());
    let walker = WalkBuilder::new(&args.path)
        .hidden(false)
        .git_ignore(true)
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    let path = entry.path();

                    if let Some(ref exts) = extensions {
                        if !file_extension_matches(path, exts) {
                            continue;
                        }
                    }

                    if is_likely_binary(path) {
                        continue;
                    }

                    if let Some(ref regexes) = excludes {
                        if regexes
                            .iter()
                            .any(|re| re.is_match(path.to_str().unwrap_or("")))
                        {
                            continue;
                        }
                    }

                    match fs::read_to_string(path) {
                        Ok(contents) => {
                            if !contents.is_empty() {
                                if let Some(ref regex) = pattern {
                                    print_file_contents_with_context(
                                        path,
                                        &contents,
                                        regex,
                                        args.context_lines,
                                    );
                                } else {
                                    print_file_contents(path, &contents);
                                }
                            }
                        }
                        Err(e) => eprintln!("Error reading file {}: {}", path.display(), e),
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

fn print_file_contents(path: &std::path::Path, contents: &str) {
    println!("### File: {}", path.display());
    println!("```");
    println!("{}", contents);
    println!("```");
    println!();
}

fn print_file_contents_with_context(
    path: &std::path::Path,
    contents: &str,
    regex: &Regex,
    context_lines: usize,
) {
    println!("### File: {}", path.display());

    let lines: Vec<&str> = contents.lines().collect();
    let mut printed_something = false;

    for (i, line) in lines.iter().enumerate() {
        if let Some(captures) = regex.captures(line) {
            printed_something = true;
            println!("Match at line {}:", i + 1);

            let start = i.saturating_sub(context_lines);
            let end = (i + context_lines + 1).min(lines.len());

            println!("```");
            for (j, context_line) in lines[start..end].iter().enumerate() {
                let line_number = start + j + 1;
                if line_number == i + 1 {
                    println!("{}: > {}", line_number, context_line);
                } else {
                    println!("{}:   {}", line_number, context_line);
                }
            }
            println!("```");

            println!("Captured:");
            for (j, capture) in captures.iter().skip(1).enumerate() {
                if let Some(c) = capture {
                    println!("  Group {}: {}", j + 1, c.as_str());
                }
            }
            println!();
        }
    }

    if !printed_something {
        println!("No matches found in this file.");
        println!();
    }
}

fn print_git_diff(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    let repo = open_repo(&args.path)?;

    let from_rev = args.git_from.as_deref().unwrap_or("HEAD");
    let to_rev = args.git_to.as_deref().unwrap_or("HEAD");

    println!("### Git diff from {} to {}", from_rev, to_rev);

    let from_obj = find_revision(&repo, from_rev)?;
    let to_obj = find_revision(&repo, to_rev)?;
    let from_tree = find_tree(&repo, from_obj, &mut buf1)?;
    let to_tree = find_tree(&repo, to_obj, &mut buf2)?;
    let changes = diff_trees(&repo, from_tree, to_tree)?;

    let pattern = args.pattern.as_ref().map(|p| Regex::new(p).unwrap());
    let extensions: Option<Vec<String>> = args
        .extensions
        .as_ref()
        .map(|exts| exts.iter().map(|e| e.to_lowercase()).collect());

    let excludes: Option<Vec<Regex>> = args
        .excludes
        .as_ref()
        .map(|patterns| patterns.iter().map(|p| Regex::new(p).unwrap()).collect());

    for change in changes {
        match change {
            Change::Addition {
                entry_mode,
                oid,
                path,
            } => {
                if let Err(e) = process_change(
                    &repo,
                    GitPath::from(&path),
                    &extensions,
                    &pattern,
                    entry_mode,
                    oid,
                    "+",
                    None,
                    &excludes,
                ) {
                    eprintln!("Error processing addition for {:?}: {}", path, e);
                }
            }
            Change::Deletion {
                entry_mode,
                oid,
                path,
            } => {
                if let Err(e) = process_change(
                    &repo,
                    GitPath::from(&path),
                    &extensions,
                    &pattern,
                    entry_mode,
                    oid,
                    "-",
                    None,
                    &excludes,
                ) {
                    eprintln!("Error processing deletion for {:?}: {}", path, e);
                }
            }
            Change::Modification {
                entry_mode,
                oid,
                path,
                previous_entry_mode,
                previous_oid,
            } => {
                if let Err(e) = process_change(
                    &repo,
                    GitPath::from(&path),
                    &extensions,
                    &pattern,
                    previous_entry_mode,
                    previous_oid,
                    "-",
                    None,
                    &excludes,
                ) {
                    eprintln!("Error processing modification (old) for {:?}: {}", path, e);
                }
                if let Err(e) = process_change(
                    &repo,
                    GitPath::from(&path),
                    &extensions,
                    &pattern,
                    entry_mode,
                    oid,
                    "+",
                    Some(previous_oid),
                    &excludes,
                ) {
                    eprintln!("Error processing modification (new) for {:?}: {}", path, e);
                }
            }
        }
    }

    Ok(())
}

fn process_change(
    repo: &Repository,
    path: impl AsRef<Path>,
    extensions: &Option<Vec<String>>,
    pattern: &Option<Regex>,
    entry_mode: EntryMode,
    oid: gix::ObjectId,
    prefix: &str,
    previous_oid: Option<gix::ObjectId>,
    excludes: &Option<Vec<Regex>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref exts) = extensions {
        if !file_extension_matches(path.as_ref(), exts) {
            return Ok(());
        }
    }
    if let Some(ref regexes) = excludes {
        if regexes
            .iter()
            .any(|re| re.is_match(path.as_ref().to_str().unwrap_or("")))
        {
            return Ok(());
        }
    }

    println!("OID: {}", oid);
    if let Some(prev_oid) = previous_oid {
        println!("Previous OID: {}", prev_oid);
    }
    println!("```diff");

    print_file_content(repo, oid, prefix, pattern)?;

    println!("```");
    println!();

    Ok(())
}
