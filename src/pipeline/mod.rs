use serde_json::Value;

pub mod addcount;
pub mod uniq;

/// A trait for processing a stream of JSON values.
pub trait Processor {
    /// Processes a single JSON value. Returns `Some(Value)` if it should be kept/modified,
    /// or `None` if it should be dropped (e.g. for `uniq`).
    fn process(&mut self, value: Value) -> Option<Value>;
}

/// Represents a configured pipeline of processors.
pub struct Pipeline {
    pub query: String,
    pub processors: Vec<Box<dyn Processor>>,
}

impl Pipeline {
    /// Parses a raw query string (e.g. "category:ctiscan | uniq ip.dest | addcount")
    /// Separates the API query from the local pipeline processors.
    pub fn parse(raw_query: &str) -> Self {
        let parts: Vec<&str> = raw_query.split('|').collect();
        let query = parts[0].trim().to_string();

        let mut processors: Vec<Box<dyn Processor>> = Vec::new();

        for part in parts.into_iter().skip(1) {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut iter = part.split_whitespace();
            let proc_name = iter.next().unwrap_or("").to_lowercase();
            let args: Vec<String> = iter.map(|s| s.to_string()).collect();

            match proc_name.as_str() {
                "uniq" => {
                    let fields = args.join("");
                    let keys: Vec<String> = fields.split(',').map(|s| s.to_string()).collect();
                    processors.push(Box::new(uniq::Uniq::new(keys)));
                }
                "addcount" => {
                    processors.push(Box::new(addcount::Addcount::new()));
                }
                _ => {
                    eprintln!("Warning: unknown pipeline processor '{}'", proc_name);
                }
            }
        }

        Self { query, processors }
    }

    /// Run the parsed pipeline transformations on a dataset.
    pub fn process_all(&mut self, results: Vec<Value>) -> Vec<Value> {
        if self.processors.is_empty() {
            return results;
        }

        let mut output = Vec::new();
        for val in results {
            let mut current_val = Some(val);
            for p in &mut self.processors {
                if let Some(v) = current_val {
                    current_val = p.process(v);
                } else {
                    break;
                }
            }
            if let Some(v) = current_val {
                output.push(v);
            }
        }
        output
    }
}
