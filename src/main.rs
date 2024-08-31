use clap::Parser;
use gix::bstr::BString;
use gix::bstr::ByteSlice;
use gix::diff::tree::recorder::Change;
use gix::diff::tree::Changes;
use gix::diff::tree::Recorder;
use gix::diff::tree::State;
use gix::objs::tree::EntryMode;
use gix::objs::Find;
use gix::objs::TreeRefIter;
use gix::Repository;
use ignore::WalkBuilder;
use regex::Regex;
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,

    #[arg(short, long)]
    pattern: Option<String>,

    #[arg(short, long, value_delimiter = ',')]
    extensions: Option<Vec<String>>,

    #[arg(short, long, default_value = "3")]
    context_lines: usize,

    #[arg(long, help = "Git revision (tag, branch, or commit) to diff from")]
    git_from: Option<String>,

    #[arg(long, help = "Git revision (tag, branch, or commit) to diff to")]
    git_to: Option<String>,
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

fn is_likely_binary(path: &std::path::Path) -> bool {
    let extension = path
        .extension()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "pdf" | "doc" | "docx" | "xls"
        | "xlsx" | "ppt" | "pptx" | "zip" | "tar" | "gz" | "7z" | "rar" | "exe" | "dll" | "so"
        | "dylib" | "mp3" | "mp4" | "avi" | "mov" | "flv" | "db" | "sqlite" => true,
        _ => false,
    }
}

fn file_extension_matches(path: impl AsRef<Path>, extensions: &[String]) -> bool {
    let extension = path
        .as_ref()
        .extension()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("");

    extensions.iter().any(|ext| ext == extension)
}

fn repo(dir: impl AsRef<Path>) -> Result<Repository, Box<dyn std::error::Error>> {
    let git = gix::open::Options::isolated()
        .filter_config_section(|_| false)
        .open(dir.as_ref())?;

    Ok(git.to_thread_local())
}

fn find_revision<'a>(
    repo: &'a Repository,
    revision_name: &str,
) -> Result<gix::Object<'a>, Box<dyn std::error::Error>> {
    match repo.rev_parse_single(revision_name) {
        Ok(id) => repo.find_object(id).map_err(|e| {
            format!(
                "Failed to find object for revision '{}': {}",
                revision_name, e
            )
            .into()
        }),
        Err(e) => Err(format!("Failed to resolve revision '{}': {}", revision_name, e).into()),
    }
}

fn find_tree<'a>(
    repo: &'a Repository,
    obj: gix::Object<'a>,
    buf: &'a mut Vec<u8>,
) -> Result<TreeRefIter<'a>, Box<dyn std::error::Error>> {
    let db = &repo.objects;
    let tree = obj.peel_to_tree()?;
    let tree_id = tree.id();
    let data = db.try_find(&tree_id, buf).unwrap().unwrap();
    let tree = data.try_into_tree_iter().unwrap();
    Ok(tree)
}

fn diff_tags<'a>(
    repo: &'a Repository,
    previous_tree: TreeRefIter,
    current_tree: TreeRefIter,
) -> Result<Vec<Change>, Box<dyn std::error::Error>> {
    let db = &repo.objects;

    let mut recorder = Recorder::default();
    Changes::from(previous_tree).needed_to_obtain(
        current_tree,
        &mut State::default(),
        db,
        &mut recorder,
    )?;
    Ok(recorder.records)
}

fn print_git_diff(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    let repo = repo(&args.path)?;

    let from_rev = args.git_from.as_deref().unwrap_or("HEAD");
    let to_rev = args.git_to.as_deref().unwrap_or("HEAD");

    println!("### Git diff from {} to {}", from_rev, to_rev);

    let from_obj = find_revision(&repo, from_rev)?;
    let to_obj = find_revision(&repo, to_rev)?;
    let from_tree = find_tree(&repo, from_obj, &mut buf1)?;
    let to_tree = find_tree(&repo, to_obj, &mut buf2)?;
    let changes = diff_tags(&repo, from_tree, to_tree)?;

    let pattern = args.pattern.as_ref().map(|p| Regex::new(p).unwrap());
    let extensions: Option<Vec<String>> = args
        .extensions
        .as_ref()
        .map(|exts| exts.iter().map(|e| e.to_lowercase()).collect());

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
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref exts) = extensions {
        if !file_extension_matches(path.as_ref(), exts) {
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

fn print_file_content(
    repo: &Repository,
    oid: gix::ObjectId,
    prefix: &str,
    pattern: &Option<Regex>,
) -> Result<(), Box<dyn std::error::Error>> {
    let object = repo.find_object(oid)?;
    let content = object.data.as_slice();

    let mut start = 0;
    while start < content.len() {
        let end = content[start..]
            .iter()
            .position(|&b| b == b'\n')
            .map_or(content.len(), |i| start + i);
        let line = &content[start..end];

        match std::str::from_utf8(line) {
            Ok(utf8_line) => {
                if let Some(ref regex) = pattern {
                    if regex.is_match(utf8_line) {
                        println!("{}{}", prefix, utf8_line);
                    }
                } else {
                    println!("{}{}", prefix, utf8_line);
                }
            }
            Err(_) => {
                println!("{}[Non-UTF-8 data: {}]", prefix, hex::encode(line));
            }
        }

        start = end + 1;
    }

    Ok(())
}
