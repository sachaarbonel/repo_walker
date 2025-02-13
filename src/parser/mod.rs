use std::str::FromStr;
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Debug)]
pub enum SupportedLanguage {
    Rust,
    JavaScript,
    Go,
}

impl FromStr for SupportedLanguage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rs" | "rust" => Ok(SupportedLanguage::Rust),
            "js" | "javascript" => Ok(SupportedLanguage::JavaScript),
            "go" => Ok(SupportedLanguage::Go),
            _ => Err(format!("Unsupported language: {}", s)),
        }
    }
}

pub struct CodeParser {
    parser: Parser,
}

impl CodeParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).expect("Error loading Rust grammar");
        Self { parser }
    }

    pub fn set_language(&mut self, lang: SupportedLanguage) -> Result<(), String> {
        let language = match lang {
            SupportedLanguage::Rust => tree_sitter_rust::language(),
            SupportedLanguage::JavaScript => tree_sitter_javascript::language(),
            SupportedLanguage::Go => tree_sitter_go::language(),
        };

        self.parser.set_language(language)
            .map_err(|e| format!("Error setting language: {}", e))
    }

    fn get_comment_query(lang: &SupportedLanguage) -> &'static str {
        match lang {
            SupportedLanguage::Rust => "(
                [(line_comment) (block_comment)] @comment
            )",
            SupportedLanguage::JavaScript => "(
                (comment) @comment
            )",
            SupportedLanguage::Go => "(
                (comment) @comment
            )",
        }
    }

    pub fn remove_comments(&mut self, source_code: &str) -> String {
        let tree = self.parser.parse(source_code, None)
            .expect("Failed to parse code");

        // Get the current language
        let lang = match self.parser.language().unwrap() {
            lang if lang == tree_sitter_rust::language() => SupportedLanguage::Rust,
            lang if lang == tree_sitter_javascript::language() => SupportedLanguage::JavaScript,
            lang if lang == tree_sitter_go::language() => SupportedLanguage::Go,
            _ => return source_code.to_string(),
        };

        // Query to match comments based on language
        let query = Query::new(self.parser.language().unwrap(), Self::get_comment_query(&lang))
            .expect("Failed to create query");

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

        // Collect all comment ranges
        let mut comment_ranges: Vec<(usize, usize)> = matches
            .flat_map(|m| {
                m.captures.iter().map(|capture| {
                    let node = capture.node;
                    (node.start_byte(), node.end_byte())
                })
            })
            .collect();

        // Sort ranges by start position
        comment_ranges.sort_by_key(|&(start, _)| start);

        // Merge overlapping ranges
        if !comment_ranges.is_empty() {
            let mut merged = Vec::new();
            let mut current = comment_ranges[0];

            for &(start, end) in comment_ranges.iter().skip(1) {
                if start <= current.1 {
                    current.1 = current.1.max(end);
                } else {
                    merged.push(current);
                    current = (start, end);
                }
            }
            merged.push(current);
            comment_ranges = merged;
        }

        // Build result string excluding comments
        let mut result = String::new();
        let mut last_end = 0;

        for (start, end) in comment_ranges {
            result.push_str(&source_code[last_end..start]);
            last_end = end;
        }

        result.push_str(&source_code[last_end..]);
        result
    }
}

impl Default for CodeParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_comment_removal() {
        let mut parser = CodeParser::new();
        let code = r#"
// Line comment
fn main() {
    /* Block comment */
    println!("Hello"); // End of line comment
    /* Multi
       line
       comment */
}
"#;
        let result = parser.remove_comments(code);
        assert!(!result.contains("// Line comment"));
        assert!(!result.contains("/* Block comment */"));
        assert!(!result.contains("// End of line comment"));
        assert!(result.contains("fn main()"));
        assert!(result.contains("println!(\"Hello\");"));
    }

    #[test]
    fn test_javascript_comment_removal() {
        let mut parser = CodeParser::new();
        parser.set_language(SupportedLanguage::JavaScript).unwrap();
        let code = r#"
// Line comment
function main() {
    /* Block comment */
    console.log("Hello"); // End of line comment
    /* Multi
       line
       comment */
}
"#;
        let result = parser.remove_comments(code);
        assert!(!result.contains("// Line comment"));
        assert!(!result.contains("/* Block comment */"));
        assert!(!result.contains("// End of line comment"));
        assert!(result.contains("function main()"));
        assert!(result.contains("console.log(\"Hello\");"));
    }

    #[test]
    fn test_go_comment_removal() {
        let mut parser = CodeParser::new();
        parser.set_language(SupportedLanguage::Go).unwrap();
        let code = r#"
// Line comment
package main

func main() {
    /* Block comment */
    fmt.Println("Hello") // End of line comment
    /* Multi
       line
       comment */
}
"#;
        let result = parser.remove_comments(code);
        assert!(!result.contains("// Line comment"));
        assert!(!result.contains("/* Block comment */"));
        assert!(!result.contains("// End of line comment"));
        assert!(result.contains("package main"));
        assert!(result.contains("func main()"));
        assert!(result.contains("fmt.Println(\"Hello\")"));
    }
} 