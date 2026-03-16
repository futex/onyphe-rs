pub mod api;
pub mod cli;
pub mod config;
pub mod pipeline;
pub mod ui;

use api::client::Client;
use clap::Parser;
use cli::args::{Cli, Commands};
use colored::Colorize;
use config::Config;
use std::collections::HashMap;
use ui::printer::print_results;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut config = Config::load(Some(&cli.config));

    // Override with CLI flags
    if let Some(key) = cli.key {
        config.api_key = Some(key);
    }
    if let Some(size) = cli.size {
        config.api_size = Some(size);
    }
    if let Some(maxpage) = cli.maxpage {
        config.api_maxpage = Some(maxpage);
    }

    let client = match Client::new(config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} {}", "Error initializing client:".red().bold(), e);
            std::process::exit(1);
        }
    };

    let mut pipe = pipeline::Pipeline {
        query: String::new(),
        processors: vec![],
    };

    let command = match cli.command {
        Commands::Search { query } => {
            pipe = pipeline::Pipeline::parse(&query);
            Commands::Search {
                query: pipe.query.clone(),
            }
        }
        Commands::Summary { kind, query } => {
            pipe = pipeline::Pipeline::parse(&query);
            Commands::Summary {
                kind,
                query: pipe.query.clone(),
            }
        }
        Commands::Simple { category, query } => {
            pipe = pipeline::Pipeline::parse(&query);
            Commands::Simple {
                category,
                query: pipe.query.clone(),
            }
        }
        Commands::SimpleBest { category, query } => {
            pipe = pipeline::Pipeline::parse(&query);
            Commands::SimpleBest {
                category,
                query: pipe.query.clone(),
            }
        }
        Commands::Export { query } => {
            pipe = pipeline::Pipeline::parse(&query);
            Commands::Export {
                query: pipe.query.clone(),
            }
        }
        Commands::User => Commands::User,
    };

    let result = match command {
        Commands::User => match client.user().await {
            Ok(res) => Ok(res.results.unwrap_or_default()),
            Err(e) => Err(e),
        },
        Commands::Search { query } => {
            let page = 1; // You could add pagination loop here based on maxpage
            match client.search(&query, page).await {
                Ok(res) => Ok(res.results.unwrap_or_default()),
                Err(e) => Err(e),
            }
        }
        // We can add other commands later, just stub them out or implement similar to search
        Commands::Summary { kind, query } => {
            let kind_str = match kind {
                cli::args::SummaryKind::Ip => "ip",
                cli::args::SummaryKind::Domain => "domain",
                cli::args::SummaryKind::Hostname => "hostname",
            };
            let path = format!("/summary/{}/{}", kind_str, urlencoding::encode(&query));
            match client.get(&path, HashMap::new()).await {
                Ok(res) => Ok(res.results.unwrap_or_default()),
                Err(e) => Err(e),
            }
        }
        Commands::Simple { category, query } => {
            let path = format!(
                "/simple/{}/{}",
                urlencoding::encode(&category),
                urlencoding::encode(&query)
            );
            match client.get(&path, HashMap::new()).await {
                Ok(res) => Ok(res.results.unwrap_or_default()),
                Err(e) => Err(e),
            }
        }
        Commands::SimpleBest { category, query } => {
            let path = format!(
                "/simple/{}/best/{}",
                urlencoding::encode(&category),
                urlencoding::encode(&query)
            );
            match client.get(&path, HashMap::new()).await {
                Ok(res) => Ok(res.results.unwrap_or_default()),
                Err(e) => Err(e),
            }
        }
        Commands::Export { query } => {
            let path = format!("/export/{}", urlencoding::encode(&query));
            match client.get(&path, HashMap::new()).await {
                Ok(res) => Ok(res.results.unwrap_or_default()),
                Err(e) => Err(e),
            }
        }
    };

    match result {
        Ok(results) => {
            let final_results = pipe.process_all(results);
            print_results(&final_results, cli.aslines);
        }
        Err(e) => {
            eprintln!("{} {}", "API Request Failed:".red().bold(), e);
            std::process::exit(1);
        }
    }

    Ok(())
}
