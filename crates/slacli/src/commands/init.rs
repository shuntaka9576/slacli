use std::io::{self, Write};

use crate::config::{Config, Credentials};
use crate::error::AppError;

pub fn execute() -> Result<(), AppError> {
    let mut config = Config::load()?;
    let mut credentials = Credentials::load()?;

    // Profile name
    eprint!("Profile name: ");
    io::stderr().flush().ok();
    let mut profile_name = String::new();
    io::stdin()
        .read_line(&mut profile_name)
        .map_err(|e| AppError::Config(format!("Failed to read input: {e}")))?;
    let profile_name = profile_name.trim().to_string();
    if profile_name.is_empty() {
        return Err(AppError::Validation(
            "Profile name is required.".to_string(),
        ));
    }

    // Get or create profile config
    let profiles = config.profiles.get_or_insert_with(Default::default);
    let profile_config = profiles
        .entry(profile_name.clone())
        .or_insert_with(Default::default);

    // Profile description
    let current_desc = profile_config.description.as_deref().unwrap_or("");
    if current_desc.is_empty() {
        eprint!("Profile description (optional): ");
    } else {
        eprint!("Profile description (optional) [{current_desc}]: ");
    }
    io::stderr().flush().ok();
    let mut desc_input = String::new();
    io::stdin()
        .read_line(&mut desc_input)
        .map_err(|e| AppError::Config(format!("Failed to read input: {e}")))?;
    let desc_input = desc_input.trim();
    if !desc_input.is_empty() {
        profile_config.description = Some(desc_input.to_string());
    }

    // Set default_profile if not set
    if config.default_profile.is_none() {
        config.default_profile = Some(profile_name.clone());
    }

    // Tokens
    let cred_profile = credentials
        .profiles
        .entry(profile_name.clone())
        .or_insert_with(Default::default);

    let user_input = rpassword::prompt_password("User token (xoxp-...): ")
        .map_err(|e| AppError::Config(format!("Failed to read input: {e}")))?;

    if !user_input.is_empty() {
        if !user_input.starts_with("xoxp-") {
            return Err(AppError::Validation(
                "User token must start with 'xoxp-'.".to_string(),
            ));
        }
        cred_profile.user_token = Some(user_input);
    }

    let bot_input = rpassword::prompt_password("Bot token (xoxb-...): ")
        .map_err(|e| AppError::Config(format!("Failed to read input: {e}")))?;

    if !bot_input.is_empty() {
        if !bot_input.starts_with("xoxb-") {
            return Err(AppError::Validation(
                "Bot token must start with 'xoxb-'.".to_string(),
            ));
        }
        cred_profile.bot_token = Some(bot_input);
    }

    config.save()?;
    credentials.save()?;

    let config_path = Config::config_path();
    let creds_path = Credentials::credentials_path();
    eprintln!("Config saved to {}", config_path.display());
    eprintln!("Credentials saved to {}", creds_path.display());

    Ok(())
}
