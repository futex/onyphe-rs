use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse {
    pub count: Option<Value>,
    pub error: Option<Value>,
    pub max_page: Option<Value>,
    pub status: String,
    pub text: Option<String>,
    pub page: Option<Value>,
    pub total: Option<Value>,
    pub results: Option<Vec<Value>>,
}
