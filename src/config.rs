use std::{num::NonZeroUsize, sync::atomic::AtomicUsize};

use crate::args::{Args, Type};

pub struct Config {
    pub kind: Option<Type>,
    pub show_hidden: bool,
    pub max_depth: Option<usize>,
    pub case_sensitive: bool,
    pub threads: usize,
    pub max_results: Option<usize>,
    pub total_results: AtomicUsize,
}

impl Config {
    pub fn build(args: Args) -> Self {
        let case_sensitive = args.case_sensitive ||
            has_uppercase_char(&args.pattern);
        let threads = args.threads.unwrap_or_else(num_of_threads).get();
        let total_results = AtomicUsize::new(0);

        Self {
            kind: args.kind,
            show_hidden: args.show_hidden,
            max_depth: args.max_depth,
            max_results: args.max_results,
            case_sensitive,
            threads,
            total_results,
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