use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub path: PathBuf,

    #[arg(short, long)]
    pub pattern: Option<String>,

    #[arg(short, long, value_delimiter = ',')]
    pub extensions: Option<Vec<String>>,

    #[arg(short, long, default_value = "3")]
    pub context_lines: usize,

    #[arg(long, help = "Git revision (tag, branch, or commit) to diff from")]
    pub git_from: Option<String>,

    #[arg(long, help = "Git revision (tag, branch, or commit) to diff to")]
    pub git_to: Option<String>,

    #[arg(long, value_delimiter = ',', help = "Patterns to exclude from the results")]
    pub excludes: Option<Vec<String>>,
}
