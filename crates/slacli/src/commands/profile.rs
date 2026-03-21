use serde_json::json;

use crate::client::SlackClient;
use crate::config::{Config, Credentials};
use crate::error::AppError;
use crate::state;

pub async fn edit(fields: Vec<(String, String)>, profile: Option<&str>) -> Result<(), AppError> {
    if fields.is_empty() {
        return Err(AppError::Validation(
            "At least one --set field must be specified.".to_string(),
        ));
    }

    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let profile_name = config.resolve_profile_name(profile)?;
    let resolved = config.resolve_profile(&credentials, profile)?;
    let token = resolved.user_token()?;
    let client = SlackClient::new();

    let mut profile_data = serde_json::Map::new();

    for (key, value) in &fields {
        if let Ok(n) = value.parse::<u64>() {
            profile_data.insert(key.clone(), json!(n));
        } else {
            profile_data.insert(key.clone(), json!(value));
        }
    }

    let body = json!({
        "profile": profile_data,
    });

    let resp = client.post_json("users.profile.set", token, &body).await?;

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
    if let Err(e) = state::append_log("profile-edit", &log_entry) {
        eprintln!("{}", e.to_json());
    }

    Ok(())
}
