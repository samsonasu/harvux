use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};

const BASE_URL: &str = "https://api.harvestapp.com/v2";
const APP_USER_AGENT: &str = "Harvux (https://github.com/bsamson/harvux)";

#[derive(Clone)]
pub struct HarvestClient {
    http: reqwest::Client,
    base_url: String,
}

impl HarvestClient {
    pub fn new(access_token: &str, account_id: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {access_token}"))
                .context("Invalid access token")?,
        );
        headers.insert(
            "Harvest-Account-Id",
            HeaderValue::from_str(account_id).context("Invalid account ID")?,
        );
        headers.insert(USER_AGENT, HeaderValue::from_static(APP_USER_AGENT));

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            http,
            base_url: BASE_URL.to_string(),
        })
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.http.get(format!("{}{}", self.base_url, path))
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.http.post(format!("{}{}", self.base_url, path))
    }

    pub fn patch(&self, path: &str) -> reqwest::RequestBuilder {
        self.http.patch(format!("{}{}", self.base_url, path))
    }

    pub fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        self.http.delete(format!("{}{}", self.base_url, path))
    }
}
