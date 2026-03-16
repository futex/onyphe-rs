use crate::pipeline::Processor;
use serde_json::{Map, Value};
use std::collections::HashSet;

pub struct Uniq {
    keys: Vec<String>,
    seen: HashSet<String>,
}

impl Uniq {
    pub fn new(keys: Vec<String>) -> Self {
        Self {
            keys,
            seen: HashSet::new(),
        }
    }

    /// Helper to extract string values for nested paths like `ip.dest`
    fn extract_value(value: &Value, path: &str) -> Option<String> {
        let mut current = value;
        for part in path.split('.') {
            if let Some(obj) = current.as_object() {
                current = obj.get(part)?;
            } else {
                return None;
            }
        }

        match current {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }

    /// Helper to construct a new JSON Object containing only the requested fields
    fn keep_fields(original: &Value, keys: &[String]) -> Value {
        let mut new_obj = Map::new();

        for key in keys {
            let parts: Vec<&str> = key.split('.').collect();

            // Extract the value from the original object
            let mut current = original;
            let mut found = true;
            for part in &parts {
                if let Some(obj) = current.as_object() {
                    if let Some(val) = obj.get(*part) {
                        current = val;
                    } else {
                        found = false;
                        break;
                    }
                } else {
                    found = false;
                    break;
                }
            }

            if found {
                // Insert the extracted value into the new object, recreating nested structure
                let mut target = &mut new_obj;
                for (i, &part) in parts.iter().enumerate() {
                    if i == parts.len() - 1 {
                        target.insert(part.to_string(), current.clone());
                    } else {
                        if !target.contains_key(part) {
                            target.insert(part.to_string(), Value::Object(Map::new()));
                        }
                        // Move `target` reference down a level
                        let next = target.get_mut(part).unwrap();
                        if let Value::Object(next_map) = next {
                            target = next_map;
                        } else {
                            break; // Should not happen in well-formed data
                        }
                    }
                }
            }
        }

        Value::Object(new_obj)
    }
}

impl Processor for Uniq {
    fn process(&mut self, value: Value) -> Option<Value> {
        if self.keys.is_empty() {
            return Some(value);
        }

        let mut signature = String::new();
        for key in &self.keys {
            if let Some(val) = Self::extract_value(&value, key) {
                signature.push_str(key);
                signature.push('-');
                signature.push_str(&val);
                signature.push(':');
            }
        }

        if signature.is_empty() {
            // Nothing to group by found on this document, keep by default or drop?
            return Some(value);
        }

        if self.seen.contains(&signature) {
            None
        } else {
            self.seen.insert(signature);
            Some(Self::keep_fields(&value, &self.keys))
        }
    }
}
