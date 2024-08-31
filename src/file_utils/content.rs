use std::path::Path;
use gix::Repository;
use regex::Regex;

pub fn file_extension_matches(path: impl AsRef<Path>, extensions: &[String]) -> bool {
    let extension = path
        .as_ref()
        .extension()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("");

    extensions.iter().any(|ext| ext == extension)
}

pub fn is_likely_binary(path: &std::path::Path) -> bool {
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

pub fn print_file_content(
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
