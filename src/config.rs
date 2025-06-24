use std::num::NonZeroUsize;

use crate::args::{Args, Type};

pub struct Config {
    pub pattern: String,
    pub kind: Option<Type>,
    pub show_hidden: bool,
    pub max_depth: Option<usize>,
    // pub count_enabled: bool,
    pub case_sensitive: bool,
    pub threads: usize,
}

impl Config {
    pub fn build(args: Args) -> Self {
        let case_sensitive = args.case_sensitive ||
        has_uppercase_char(&args.pattern);
    
        let threads = args.threads.unwrap_or_else(num_of_threads).get();
        
        Self {
            pattern: args.pattern,
            kind: args.kind,
            show_hidden: args.show_hidden,
            max_depth: args.max_depth,
            // count_enabled: args.count_enabled,
            case_sensitive,
            threads,
        }
    }
}

fn num_of_threads() -> NonZeroUsize {
    std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::MIN)
}

fn has_uppercase_char(pattern: &str) -> bool {
    pattern.chars().any(|c| c.is_uppercase())
}