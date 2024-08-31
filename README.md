# Repo Walker

Repo Walker is a command-line tool for analyzing Git repositories and formatting the output for easy use with Large Language Models (LLMs) like Anthropic's Claude. It's designed to work seamlessly with clipboard utilities like `pbcopy`, allowing you to quickly provide context to AI assistants that are limited in the number of documents you can upload.

## Features

- Compare changes between two Git tags, branches or commits
- Filter files by extension
- Apply regex pattern matching to file contents
- Handle non-UTF-8 file contents
- Format output suitable for AI assistants
- Easy integration with clipboard utilities for use with LLMs

## Installation

Ensure you have Rust and Cargo installed on your system. Then, clone this repository and build the project:

```bash
cargo install --git https://github.com/sachaarbonel/repo_walker.git
```

## Usage

### Basic Usage

```bash
repo_walker --path /path/to/repo [OPTIONS] | pbcopy
```

This command will analyze the repository and copy the output to your clipboard, ready to paste into an AI assistant chat.

### Compare two tags, branches or commits

```bash
repo_walker --path /path/to/repo --git-from v1.0 --git-to v2.0 [OPTIONS] | pbcopy
```

### Options

- `--path <PATH>`: Path to the Git repository (required)
- `--git-from <REVISION>`: Starting tag, commit or branch for comparison
- `--git-to <REVISION>`: Ending tag,commit or branch for for comparison
- `--extensions <EXT1,EXT2,...>`: Comma-separated list of file extensions to include
- `--pattern <REGEX>`: Regex pattern to filter file contents
- `--context-lines <NUM>`: Number of context lines to show (default: 3)

## Examples

1. Compare two tags, branches or commits, showing only Rust files, and copy to clipboard:
   ```bash
   repo_walker --path /path/to/repo --git-from v0.1.0 --git-to v0.2.0 --extensions rs | pbcopy
   ```

2. Compare tags, branches or commits with more context lines and copy to clipboard:
   ```bash
   repo_walker --path /path/to/repo --git-from v1.0 --git-to v2.0 --context-lines 5 | pbcopy
   ```

## Integration with AI Assistants

Repo Walker is particularly useful when working with AI assistants that don't support direct file uploads, such as Anthropic's Claude. Here's how to use it:

1. Run Repo Walker with your desired options, piping the output to `pbcopy`.
2. In your conversation with the AI assistant, paste the clipboard content.
3. The AI can now analyze the repository changes and provide insights or answer questions based on the code context.

This workflow allows you to quickly provide substantial code context to the AI without the need for manual copying or formatting.

## Output

The tool outputs the diff in a format suitable for pasting into AI assistant chats:

```
### Git diff from v0.1.0 to v0.2.0
File: src/main.rs
Mode: 100644
OID: 1234567890abcdef1234567890abcdef12345678
```diff
+fn new_function() {
+    println!("This is a new function");
+}
-fn old_function() {
-    println!("This function will be removed");
-}
```

## Error Handling

- The tool handles non-UTF-8 file contents by displaying them as hexadecimal.
- If a file cannot be processed, an error message is displayed, and the tool continues with the next file.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
