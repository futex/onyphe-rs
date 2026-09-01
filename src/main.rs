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

fn normalize_args<I: IntoIterator<Item = String>>(args: I) -> Vec<String> {
    let mut normalized = Vec::new();
    let mut iter = args.into_iter();
    if let Some(bin) = iter.next() {
        normalized.push(bin);
    }
    let mut in_flags = true;
    for arg in iter {
        if !in_flags {
            normalized.push(arg);
            continue;
        }

        if arg == "--" {
            in_flags = false;
            normalized.push(arg);
            continue;
        }

        if arg.starts_with('-') && !arg.starts_with("--") {
            let (key, val) = match arg.split_once('=') {
                Some((k, v)) => (k, Some(v)),
                None => (arg.as_str(), None),
            };

            let norm_key = match key {
                "-size" => "--size",
                "-key" => "--key",
                "-maxpage" => "--maxpage",
                "-config" => "--config",
                "-aslines" => "--aslines",
                "-verbose" => "--verbose",
                "-user" => "user",
                "-search" => "search",
                "-summary" => "summary",
                "-simple" => "simple",
                "-simple-best" | "-simplebest" => "simple-best",
                "-export" => "export",
                "-help" => "--help",
                "-version" => "--version",
                _ => key,
            };

            if let Some(v) = val {
                if norm_key.starts_with("--") {
                    normalized.push(format!("{}={}", norm_key, v));
                } else {
                    normalized.push(norm_key.to_string());
                    normalized.push(v.to_string());
                }
            } else {
                normalized.push(norm_key.to_string());
            }
        } else {
            normalized.push(arg);
        }
    }
    normalized
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse_from(normalize_args(std::env::args()));

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

#[cfg(test)]
mod tests {
    use super::*;
    use cli::args::Commands;

    #[test]
    fn test_size_after_subcommand_double_dash() {
        let raw = vec!["onyphe", "search", "category:datascan", "--size", "10"];
        let norm = normalize_args(raw.into_iter().map(String::from));
        let cli = Cli::parse_from(norm);
        assert_eq!(cli.size, Some(10));
        match cli.command {
            Commands::Search { query } => assert_eq!(query, "category:datascan"),
            _ => panic!("Expected Search subcommand"),
        }
    }

    #[test]
    fn test_size_after_subcommand_single_dash() {
        let raw = vec!["onyphe", "search", "category:datascan", "-size", "10"];
        let norm = normalize_args(raw.into_iter().map(String::from));
        let cli = Cli::parse_from(norm);
        assert_eq!(cli.size, Some(10));
        match cli.command {
            Commands::Search { query } => assert_eq!(query, "category:datascan"),
            _ => panic!("Expected Search subcommand"),
        }
    }

    #[test]
    fn test_size_before_subcommand_single_dash() {
        let raw = vec!["onyphe", "-size", "10", "search", "category:datascan"];
        let norm = normalize_args(raw.into_iter().map(String::from));
        let cli = Cli::parse_from(norm);
        assert_eq!(cli.size, Some(10));
        match cli.command {
            Commands::Search { query } => assert_eq!(query, "category:datascan"),
            _ => panic!("Expected Search subcommand"),
        }
    }

    #[test]
    fn test_size_equals_format() {
        let raw = vec!["onyphe", "search", "category:datascan", "-size=25"];
        let norm = normalize_args(raw.into_iter().map(String::from));
        let cli = Cli::parse_from(norm);
        assert_eq!(cli.size, Some(25));
    }

    #[test]
    fn test_single_dash_subcommand() {
        let raw = vec!["onyphe", "-search", "category:datascan", "-size", "100", "-aslines"];
        let norm = normalize_args(raw.into_iter().map(String::from));
        let cli = Cli::parse_from(norm);
        assert_eq!(cli.size, Some(100));
        assert!(cli.aslines);
        match cli.command {
            Commands::Search { query } => assert_eq!(query, "category:datascan"),
            _ => panic!("Expected Search subcommand"),
        }
    }

    #[test]
    fn test_short_size_flag() {
        let raw = vec!["onyphe", "search", "category:datascan", "-s", "50"];
        let norm = normalize_args(raw.into_iter().map(String::from));
        let cli = Cli::parse_from(norm);
        assert_eq!(cli.size, Some(50));
    }

    #[test]
    fn test_all_global_options_single_dash() {
        let raw = vec![
            "onyphe",
            "search",
            "category:datascan",
            "-key",
            "APIKEY123",
            "-size",
            "100",
            "-maxpage",
            "5",
            "-config",
            "/tmp/custom.ini",
            "-aslines",
            "-verbose",
        ];
        let norm = normalize_args(raw.into_iter().map(String::from));
        let cli = Cli::parse_from(norm);
        assert_eq!(cli.key, Some("APIKEY123".to_string()));
        assert_eq!(cli.size, Some(100));
        assert_eq!(cli.maxpage, Some(5));
        assert_eq!(cli.config, "/tmp/custom.ini");
        assert!(cli.aslines);
        assert!(cli.verbose);
    }
}
