use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

pub struct Args {
    #[arg(short, long)]
    pub file: String,

    #[arg(short = 'k', long = "kind", default_value_t = Type::File)]
    pub kind: Type,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Type {
    Folder,
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