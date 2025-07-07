use crate::args::{Args, Type};
use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct Config {
    pub kind: Option<Type>,
    pub show_hidden: bool,
    pub max_depth: Option<usize>,
    pub case_sensitive: bool,
    pub threads: usize,
}

impl Config {
    pub fn build(args: Args) -> Self {
        let case_sensitive = args.case_sensitive || has_uppercase_char(&args.pattern);
        let threads = args.threads.unwrap_or_else(num_of_threads).get();

        Self {
            kind: args.kind,
            show_hidden: args.show_hidden,
            max_depth: args.max_depth,
            case_sensitive,
            threads,
        }
    }
}

fn num_of_threads() -> NonZeroUsize {
    std::thread::available_parallelism().unwrap_or(NonZeroUsize::MIN)
}

fn has_uppercase_char(pattern: &str) -> bool {
    pattern.chars().any(|c| c.is_uppercase())
}
