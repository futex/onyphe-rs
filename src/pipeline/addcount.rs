use crate::pipeline::Processor;
use serde_json::Value;

pub struct Addcount {
    count: usize,
}

impl Addcount {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl Processor for Addcount {
    fn process(&mut self, mut value: Value) -> Option<Value> {
        self.count += 1;
        if let Some(obj) = value.as_object_mut() {
            obj.insert("count".to_string(), Value::Number(self.count.into()));
        }
        Some(value)
    }
}
