use crate::config::Config;
use crate::error::AppError;

pub fn open_editor() -> Result<(), AppError> {
    let config = Config::load()?;
    let editor = config.resolve_editor();
    let path = Config::config_path();

    if !path.exists() {
        config.save()?;
    }

    let status = std::process::Command::new(&editor)
        .arg(&path)
        .status()
        .map_err(|e| AppError::Config(format!("Failed to open editor '{editor}': {e}")))?;

    if !status.success() {
        return Err(AppError::Config(format!(
            "Editor '{editor}' exited with error"
        )));
    }

    Ok(())
}

pub fn see(profile: Option<&str>) -> Result<(), AppError> {
    let config = Config::load()?;

    let output = if let Some(profile_name) = profile {
        let profile_config = config.get_profile(profile_name)?;
        serde_json::json!({
            "profile": profile_name,
            "description": profile_config.description,
            "channels": profile_config.channels,
        })
    } else {
        serde_json::json!({
            "default_profile": config.default_profile,
            "editor": config.editor,
            "profiles": config.profiles,
        })
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    Ok(())
}
