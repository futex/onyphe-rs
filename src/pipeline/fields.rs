use crate::pipeline::Processor;
use serde_json::{Map, Value};

pub struct Fields {
    keys: Vec<String>,
}

impl Fields {
    pub fn new(keys: Vec<String>) -> Self {
        Self { keys }
    }

    fn extract<'a>(src: &'a Map<String, Value>, path: &str) -> Option<&'a Value> {
        let mut parts = path.split('.');
        let mut current = src.get(parts.next()?)?;
        for part in parts {
            current = current.as_object()?.get(part)?;
        }
        Some(current)
    }

    fn insert_nested(out: &mut Map<String, Value>, path: &str, value: Value) {
        let mut target = out;
        let mut parts = path.split('.').peekable();
        while let Some(part) = parts.next() {
            if parts.peek().is_none() {
                target.insert(part.to_string(), value);
                return;
            }
            target = match target
                .entry(part)
                .or_insert_with(|| Value::Object(Map::new()))
            {
                Value::Object(map) => map,
                _ => return,
            };
        }
    }
}

impl Processor for Fields {
    fn process(&mut self, value: Value) -> Option<Value> {
        if self.keys.is_empty() {
            return Some(value);
        }

        let src = match value {
            Value::Object(map) => map,
            other => return Some(other),
        };

        let mut out = Map::new();
        for key in &self.keys {
            if let Some(v) = src.get(key) {
                out.insert(key.clone(), v.clone());
            } else if let Some(v) = Self::extract(&src, key) {
                Self::insert_nested(&mut out, key, v.clone());
            }
        }

        if out.is_empty() {
            None
        } else {
            Some(Value::Object(out))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::Pipeline;
    use serde_json::json;

    fn keys(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_fields_keeps_only_requested_keys() {
        let mut p = Fields::new(keys(&["ip", "port"]));
        let out = p
            .process(json!({"ip": "1.2.3.4", "port": 22, "protocol": "ssh"}))
            .unwrap();
        assert_eq!(out, json!({"ip": "1.2.3.4", "port": 22}));
    }

    #[test]
    fn test_fields_preserves_requested_order() {
        let mut p = Fields::new(keys(&["protocol", "ip", "port"]));
        let out = p
            .process(json!({"ip": "1.2.3.4", "port": 22, "protocol": "ssh"}))
            .unwrap();
        let order: Vec<&str> = out.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(order, vec!["protocol", "ip", "port"]);
    }

    #[test]
    fn test_fields_ignores_missing_keys() {
        let mut p = Fields::new(keys(&["ip", "missing", "port"]));
        let out = p.process(json!({"ip": "1.2.3.4", "port": 22})).unwrap();
        assert_eq!(out, json!({"ip": "1.2.3.4", "port": 22}));
    }

    #[test]
    fn test_fields_at_prefixed_key() {
        let mut p = Fields::new(keys(&["@timestamp", "@category"]));
        let out = p
            .process(json!({"@timestamp": "2026-01-01T00:00:00Z", "@category": "datascan", "ip": "1.2.3.4"}))
            .unwrap();
        assert_eq!(
            out,
            json!({"@timestamp": "2026-01-01T00:00:00Z", "@category": "datascan"})
        );
    }

    #[test]
    fn test_fields_nested_path() {
        let mut p = Fields::new(keys(&["app.http.title", "ip"]));
        let out = p
            .process(json!({"ip": "1.2.3.4", "app": {"http": {"title": "Hello", "status": 200}}}))
            .unwrap();
        assert_eq!(out, json!({"app": {"http": {"title": "Hello"}}, "ip": "1.2.3.4"}));
    }

    #[test]
    fn test_fields_flat_dotted_key() {
        let mut p = Fields::new(keys(&["ip.dest"]));
        let out = p.process(json!({"ip.dest": "1.2.3.4", "port": 22})).unwrap();
        assert_eq!(out, json!({"ip.dest": "1.2.3.4"}));
    }

    #[test]
    fn test_fields_drops_document_without_any_field() {
        let mut p = Fields::new(keys(&["missing"]));
        assert_eq!(p.process(json!({"ip": "1.2.3.4"})), None);
    }

    #[test]
    fn test_fields_empty_list_is_noop() {
        let mut p = Fields::new(vec![]);
        let doc = json!({"ip": "1.2.3.4"});
        assert_eq!(p.process(doc.clone()), Some(doc));
    }

    #[test]
    fn test_fields_registered_in_pipeline() {
        let mut pipe = Pipeline::parse("category:datascan | fields @timestamp, ip,port");
        assert_eq!(pipe.query, "category:datascan");
        assert_eq!(pipe.processors.len(), 1);
        let out = pipe.process_all(vec![
            json!({"@timestamp": "t", "ip": "1.2.3.4", "port": 22, "protocol": "ssh"}),
        ]);
        assert_eq!(out, vec![json!({"@timestamp": "t", "ip": "1.2.3.4", "port": 22})]);
    }
}
