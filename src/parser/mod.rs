use std::str::FromStr;
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Debug)]
pub enum SupportedLanguage {
    Rust,
    JavaScript,
    Python,
    Go,
}

impl FromStr for SupportedLanguage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rs" | "rust" => Ok(SupportedLanguage::Rust),
            "js" | "javascript" => Ok(SupportedLanguage::JavaScript),
            "py" | "python" => Ok(SupportedLanguage::Python),
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
            SupportedLanguage::Python => tree_sitter_python::language(),
            SupportedLanguage::Go => tree_sitter_go::language(),
        };

        self.parser.set_language(language)
            .map_err(|e| format!("Error setting language: {}", e))
    }

    pub fn remove_comments(&mut self, source_code: &str) -> String {
        let tree = self.parser.parse(source_code, None)
            .expect("Failed to parse code");

        // Query to match comments
        let query = Query::new(self.parser.language().unwrap(),
            "(line_comment) @comment
             (block_comment) @comment")
            .expect("Failed to create query");

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

        // Collect all comment ranges
        let mut comment_ranges: Vec<(usize, usize)> = matches
            .map(|m| {
                let node = m.captures[0].node;
                (node.start_byte(), node.end_byte())
            })
            .collect();

        // Sort ranges by start position
        comment_ranges.sort_by_key(|&(start, _)| start);

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
} 