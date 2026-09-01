use reqwest::{Client as ReqwestClient, Url};
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
}
