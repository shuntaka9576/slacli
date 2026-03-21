use crate::error::AppError;
use crate::state;

pub fn run(log_type: &str, purge: bool) -> Result<(), AppError> {
    match log_type {
        "chat-send" | "profile-edit" => {
            if purge {
                state::clear_logs(log_type)
            } else {
                state::print_logs(log_type)
            }
        }
        _ => Err(AppError::Validation(format!(
            "Unknown log type: '{log_type}'. Available types: chat-send, profile-edit"
        ))),
    }
}
