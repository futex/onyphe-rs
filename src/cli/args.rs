use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "onyphe", about = "ONYPHE Command Line Interface", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbosity level
    #[arg(short, long)]
    pub verbose: bool,

    /// Config file path
    #[arg(long, default_value = "~/.onyphe.ini")]
    pub config: String,

    /// API Key
    #[arg(long)]
    pub key: Option<String>,

    /// Max page to fetch with Search API
    #[arg(long)]
    pub maxpage: Option<u32>,

    /// Fetch pages of size results
    #[arg(long)]
    pub size: Option<u32>,

    /// Output scan results as lines of JSON
    #[arg(long)]
    pub aslines: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get information on your license
    User,

    /// Use Search API for query
    Search { query: String },

    /// Use Summary API for query
    Summary {
        #[arg(value_enum)]
        kind: SummaryKind,
        query: String,
    },

    /// Use Simple API for query
    Simple { category: String, query: String },

    /// Use Simple Best API for query
    SimpleBest { category: String, query: String },

    /// Export API for query
    Export { query: String },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum SummaryKind {
    Ip,
    Domain,
    Hostname,
}
