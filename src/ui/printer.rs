use colored::Colorize;
use serde_json::Value;

pub fn print_results(results: &[Value], aslines: bool) {
    if results.is_empty() {
        if !aslines {
            println!("{}", "No results found.".yellow());
        }
        return;
    }

    for res in results {
        // We always print as JSON lines now to match expected script behavior
        println!(
            "{}",
            serde_json::to_string(res).unwrap_or_else(|_| "{}".to_string())
        );
    }
}
