use serde::Deserialize;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const DEEPSEEK_API_BASE: &str = "https://api.deepseek.com";

#[derive(Debug, Clone)]
pub struct Balance {
    pub total: f64,
    pub topped_up: f64,
    pub granted: f64,
    #[allow(dead_code)]
    pub currency: String,
}

#[derive(Debug, Deserialize)]
struct BalanceInfoRaw {
    pub currency: String,
    pub total_balance: String,
    pub topped_up_balance: String,
    pub granted_balance: String,
}

#[derive(Debug, Deserialize)]
struct BalanceResponseRaw {
    #[serde(default)]
    pub is_available: bool,
    pub balance_infos: Vec<BalanceInfoRaw>,
}

#[derive(Debug)]
pub enum ApiError {
    Network(String),
    Unauthorized,
    RateLimited,
    ClientError(u16),
    ServerError(u16),
    ParseError(String),
    ServiceUnavailable,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(msg) => write!(f, "network error: {}", msg),
            ApiError::Unauthorized => write!(f, "invalid API key"),
            ApiError::RateLimited => write!(f, "rate limited, retry later"),
            ApiError::ClientError(code) => write!(f, "client error (HTTP {})", code),
            ApiError::ServerError(code) => write!(f, "server error (HTTP {})", code),
            ApiError::ParseError(msg) => write!(f, "parse error: {}", msg),
            ApiError::ServiceUnavailable => write!(f, "balance service unavailable"),
        }
    }
}

impl std::error::Error for ApiError {}

pub async fn fetch_balance(api_key: &str) -> Result<Balance, ApiError> {
    fetch_balance_inner(api_key, DEEPSEEK_API_BASE).await
}

#[allow(dead_code)]
pub async fn fetch_balance_with_url(api_key: &str, base_url: &str) -> Result<Balance, ApiError> {
    fetch_balance_inner(api_key, base_url).await
}

async fn fetch_balance_inner(api_key: &str, base_url: &str) -> Result<Balance, ApiError> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| ApiError::Network(format!("failed to create HTTP client: {}", e)))?;

    let url = format!("{}/user/balance", base_url.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                ApiError::Network("request timeout".into())
            } else {
                ApiError::Network(format!("request failed: {}", e))
            }
        })?;

    let status = response.status().as_u16();
    match status {
        200 => {
            let raw: BalanceResponseRaw = response.json().await.map_err(|e| {
                ApiError::ParseError(format!("JSON parse failed: {}", e))
            })?;

            if !raw.is_available {
                return Err(ApiError::ServiceUnavailable);
            }

            let info = raw.balance_infos.into_iter().next().ok_or_else(|| {
                ApiError::ParseError("empty balance_infos".into())
            })?;

            let total = info.total_balance.parse::<f64>().map_err(|e| {
                ApiError::ParseError(format!(
                    "total_balance '{}' parse failed: {}",
                    info.total_balance, e
                ))
            })?;
            let topped_up = info.topped_up_balance.parse::<f64>().map_err(|e| {
                ApiError::ParseError(format!(
                    "topped_up_balance '{}' parse failed: {}",
                    info.topped_up_balance, e
                ))
            })?;
            let granted = info.granted_balance.parse::<f64>().map_err(|e| {
                ApiError::ParseError(format!(
                    "granted_balance '{}' parse failed: {}",
                    info.granted_balance, e
                ))
            })?;

            Ok(Balance { total, topped_up, granted, currency: info.currency })
        }
        401 => Err(ApiError::Unauthorized),
        429 => Err(ApiError::RateLimited),
        403 => Err(ApiError::ClientError(403)),
        s if (400..500).contains(&s) => Err(ApiError::ClientError(s)),
        s if s >= 500 => Err(ApiError::ServerError(s)),
        other => Err(ApiError::ServerError(other)),
    }
}
