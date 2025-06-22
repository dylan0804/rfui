use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

pub struct Args {
    #[arg(
        help = "Pattern to search"
    )]
    pub pattern: String,

    #[arg(
        short = 'k', 
        long = "kind", 
        default_value_t = Type::File,
        help = "Filter by type: file (f/file) or directory (d/dir)",
    )]
    pub kind: Type,

    #[arg(
        short = 'H', 
        long = "show-hidden", 
        default_value_t = false
    )]
    pub show_hidden: bool,

    #[arg(
        short = 'd',
        long = "max-depth",
        help = "Set maximum depth search"
    )]
    pub max_depth: Option<usize>,

    #[arg(
        short = 'c',
        long = "count",
        help = "Display count"
    )]
    pub count_enabled: bool
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Type {
    #[value(alias = "d", alias = "dir")]
    Directory,
    
    #[value(alias = "f", alias = "file")]
    File
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}