use serde_json::json;

use crate::client::SlackClient;
use crate::config::{Config, Credentials};
use crate::error::AppError;
use crate::state;

pub async fn send(
    channel: &str,
    text: &str,
    thread_ts: Option<&str>,
    profile: Option<&str>,
) -> Result<(), AppError> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let profile_name = config.resolve_profile_name(profile)?;
    let resolved = config.resolve_profile(&credentials, profile)?;
    let channel = resolved.resolve_channel(channel);
    let token = resolved.bot_token()?;
    let client = SlackClient::new();

    let mut body = json!({
        "channel": channel,
        "text": text,
    });
    if let Some(ts) = thread_ts {
        body.as_object_mut()
            .unwrap()
            .insert("thread_ts".to_string(), json!(ts));
    }

    let resp = client.post_json("chat.postMessage", token, &body).await?;

    println!("{}", serde_json::to_string(&resp).unwrap());

    if resp.get("ok") != Some(&serde_json::Value::Bool(true)) {
        return Err(AppError::Api(
            resp.get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("unknown error")
                .to_string(),
        ));
    }

    let mut log_entry = resp.clone();
    log_entry
        .as_object_mut()
        .unwrap()
        .insert("_slacli_profile".to_string(), json!(profile_name));
    if let Err(e) = state::append_log("chat-send", &log_entry) {
        eprintln!("{}", e.to_json());
    }

    Ok(())
}

pub async fn delete(channel: &str, timestamp: &str, profile: Option<&str>) -> Result<(), AppError> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let resolved = config.resolve_profile(&credentials, profile)?;
    let channel = resolved.resolve_channel(channel);
    let token = resolved.bot_token()?;
    let client = SlackClient::new();

    let body = json!({
        "channel": channel,
        "ts": timestamp,
    });

    let resp = client.post_json("chat.delete", token, &body).await?;

    println!("{}", serde_json::to_string(&resp).unwrap());

    if resp.get("ok") != Some(&serde_json::Value::Bool(true)) {
        return Err(AppError::Api(
            resp.get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("unknown error")
                .to_string(),
        ));
    }

    if let Err(e) = state::remove_log_entry("chat-send", &channel, timestamp) {
        eprintln!("{}", e.to_json());
    }

    Ok(())
}
