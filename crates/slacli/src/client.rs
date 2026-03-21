use serde_json::Value;

use crate::error::AppError;

const SLACK_API_BASE: &str = "https://slack.com/api";

pub struct SlackClient {
    http: reqwest::Client,
}

impl SlackClient {
    pub fn new() -> Self {
        SlackClient {
            http: reqwest::Client::new(),
        }
    }

    pub async fn post_json(
        &self,
        method: &str,
        token: &str,
        body: &Value,
    ) -> Result<Value, AppError> {
        let url = format!("{SLACK_API_BASE}/{method}");
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Request failed: {e}")))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| AppError::Network(format!("Failed to read response: {e}")))?;

        if !status.is_success() {
            return Err(AppError::Network(format!("HTTP {status}: {body}")));
        }

        serde_json::from_str(&body)
            .map_err(|e| AppError::Api(format!("Invalid JSON response: {e}")))
    }
}
