use reqwest::{Client as ReqwestClient, Url};
use serde_json::Value;
use std::collections::HashMap;

use super::models::ApiResponse;
use crate::config::Config;

pub struct Client {
    http: ReqwestClient,
    config: Config,
    endpoint: String,
    api_key: String,
}

impl Client {
    pub fn new(config: Config) -> Result<Self, String> {
        let endpoint = config
            .api_endpoint
            .clone()
            .unwrap_or_else(|| "https://www.onyphe.io/api/v2".to_string());

        let api_key = config
            .api_key
            .clone()
            .ok_or_else(|| "API Key is required. Please set it in ~/.onyphe.ini".to_string())?;

        let http = ReqwestClient::builder()
            .user_agent(concat!("onyphe-rs/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self {
            http,
            config,
            endpoint,
            api_key,
        })
    }

    pub async fn get(
        &self,
        path: &str,
        params: HashMap<&str, String>,
    ) -> Result<ApiResponse, String> {
        let mut url =
            Url::parse(&format!("{}{}", self.endpoint, path)).map_err(|e| e.to_string())?;

        let mut query_params = params;
        query_params.insert("k", self.api_key.clone());

        // Add global config params
        if let Some(size) = self.config.api_size {
            query_params.insert("size", size.to_string());
        }
        if let Some(true) = self.config.api_trackquery {
            query_params.insert("trackquery", "true".to_string());
        }
        if let Some(true) = self.config.api_calculated {
            query_params.insert("calculated", "true".to_string());
        }

        url.query_pairs_mut().extend_pairs(query_params.into_iter());

        let res = self.http.get(url).send().await.map_err(|e| e.to_string())?;

        if res.status().is_success() {
            let api_res: ApiResponse = res.json().await.map_err(|e| e.to_string())?;
            Ok(api_res)
        } else {
            let status = res.status();
            match res.json::<ApiResponse>().await {
                Ok(api_res) => Err(format!("API Error {}: {:?}", status, api_res.text)),
                Err(_) => Err(format!("HTTP Error {}", status)),
            }
        }
    }

    pub async fn user(&self) -> Result<ApiResponse, String> {
        self.get("/user", HashMap::new()).await
    }

    pub async fn search(&self, query: &str, page: u32) -> Result<ApiResponse, String> {
        let mut params = HashMap::new();
        params.insert("page", page.to_string());
        let path = format!("/search/{}", urlencoding::encode(query));
        self.get(&path, params).await
    }

    pub async fn export(&self, query: &str) -> Result<Vec<Value>, String> {
        let mut url = Url::parse(&format!(
            "{}/export/{}",
            self.endpoint,
            urlencoding::encode(query)
        ))
        .map_err(|e| e.to_string())?;

        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("k", &self.api_key);
            if let Some(true) = self.config.api_trackquery {
                pairs.append_pair("trackquery", "true");
            }
            if let Some(true) = self.config.api_calculated {
                pairs.append_pair("calculated", "true");
            }
        }

        let mut res = self.http.get(url).send().await.map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let status = res.status();
            return match res.json::<ApiResponse>().await {
                Ok(api_res) => Err(format!("API Error {}: {:?}", status, api_res.text)),
                Err(_) => Err(format!("HTTP Error {}", status)),
            };
        }

        let mut docs = Vec::new();
        let mut buf: Vec<u8> = Vec::new();
        while let Some(chunk) = res.chunk().await.map_err(|e| e.to_string())? {
            buf.extend_from_slice(&chunk);
            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                Self::push_export_line(&line, &mut docs)?;
            }
        }
        Self::push_export_line(&buf, &mut docs)?;

        Ok(docs)
    }

    fn push_export_line(line: &[u8], docs: &mut Vec<Value>) -> Result<(), String> {
        let line = String::from_utf8_lossy(line);
        let line = line.trim();
        if line.is_empty() {
            return Ok(());
        }

        let doc: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
        if doc.get("status").and_then(Value::as_str) == Some("nok") {
            return Err(format!(
                "API Error: {}",
                doc.get("text").and_then(Value::as_str).unwrap_or_default()
            ));
        }
        if doc.get("@category").and_then(Value::as_str) == Some("none") {
            return Ok(());
        }

        docs.push(doc);
        Ok(())
    }
}
