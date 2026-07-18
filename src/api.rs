use serde::Deserialize;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const DEEPSEEK_API_BASE: &str = "https://api.deepseek.com";

#[derive(Debug, Clone)]
pub struct Balance {
    pub total: f64,
    pub topped_up: f64,
    pub granted: f64,
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
    ServerError(u16),
    ParseError(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(msg) => write!(f, "网络错误: {}", msg),
            ApiError::Unauthorized => write!(f, "API Key 无效"),
            ApiError::RateLimited => write!(f, "请求过于频繁，请稍后重试"),
            ApiError::ServerError(code) => write!(f, "服务器错误 (HTTP {})", code),
            ApiError::ParseError(msg) => write!(f, "数据解析错误: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

/// Fetch balance from DeepSeek API.
pub async fn fetch_balance(api_key: &str) -> Result<Balance, ApiError> {
    fetch_balance_inner(api_key, DEEPSEEK_API_BASE).await
}

/// Fetch balance from a custom base URL (for testing with httpmock).
pub async fn fetch_balance_with_url(api_key: &str, base_url: &str) -> Result<Balance, ApiError> {
    fetch_balance_inner(api_key, base_url).await
}

async fn fetch_balance_inner(api_key: &str, base_url: &str) -> Result<Balance, ApiError> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| ApiError::Network(format!("创建 HTTP 客户端失败: {}", e)))?;

    let url = format!("{}/user/balance", base_url.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                ApiError::Network("请求超时".into())
            } else {
                ApiError::Network(format!("请求失败: {}", e))
            }
        })?;

    let status = response.status().as_u16();
    match status {
        200 => {
            let raw: BalanceResponseRaw = response
                .json()
                .await
                .map_err(|e| ApiError::ParseError(format!("JSON 解析失败: {}", e)))?;

            let info = raw
                .balance_infos
                .into_iter()
                .next()
                .ok_or_else(|| ApiError::ParseError("余额数据为空".into()))?;

            let total = info
                .total_balance
                .parse::<f64>()
                .map_err(|_| ApiError::ParseError("余额格式异常".into()))?;
            let topped_up = info
                .topped_up_balance
                .parse::<f64>()
                .map_err(|_| ApiError::ParseError("充值余额格式异常".into()))?;
            let granted = info
                .granted_balance
                .parse::<f64>()
                .map_err(|_| ApiError::ParseError("赠送余额格式异常".into()))?;

            Ok(Balance {
                total,
                topped_up,
                granted,
                currency: info.currency,
            })
        }
        401 => Err(ApiError::Unauthorized),
        429 => Err(ApiError::RateLimited),
        s if s >= 500 => Err(ApiError::ServerError(s)),
        other => Err(ApiError::ServerError(other)),
    }
}
