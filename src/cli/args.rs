use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "onyphe", about = "ONYPHE Command Line Interface", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbosity level
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,

    /// Config file path
    #[arg(short = 'c', long, global = true, default_value = "~/.onyphe.ini")]
    pub config: String,

    /// API Key
    #[arg(short = 'k', long, global = true)]
    pub key: Option<String>,

    /// Max page to fetch with Search API
    #[arg(short = 'm', long, global = true)]
    pub maxpage: Option<u32>,

    /// Fetch pages of size results
    #[arg(short = 's', long, global = true)]
    pub size: Option<u32>,

    /// Output scan results as lines of JSON
    #[arg(short = 'a', long, global = true)]
    pub aslines: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get information on your license
    #[command(alias = "-user", alias = "--user")]
    User,

    /// Use Search API for query
    #[command(alias = "-search", alias = "--search")]
    Search { query: String },

    /// Use Summary API for query
    #[command(alias = "-summary", alias = "--summary")]
    Summary {
        #[arg(value_enum)]
        kind: SummaryKind,
        query: String,
    },

    /// Use Simple API for query
    #[command(alias = "-simple", alias = "--simple")]
    Simple { category: String, query: String },

    /// Use Simple Best API for query
    #[command(name = "simple-best", alias = "simplebest", alias = "-simple-best", alias = "--simple-best", alias = "-simplebest")]
    SimpleBest { category: String, query: String },

    /// Export API for query
    #[command(alias = "-export", alias = "--export")]
    Export { query: String },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum SummaryKind {
    Ip,
    Domain,
    Hostname,
}
