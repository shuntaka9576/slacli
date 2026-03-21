use std::fmt;
use std::process;

#[derive(Debug)]
pub enum AppError {
    Config(String),
    Validation(String),
    Api(String),
    Network(String),
    Io(String),
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::Config(_) => 1,
            AppError::Validation(_) => 2,
            AppError::Api(_) => 3,
            AppError::Network(_) => 4,
            AppError::Io(_) => 5,
        }
    }

    fn error_type(&self) -> &str {
        match self {
            AppError::Config(_) => "config_error",
            AppError::Validation(_) => "validation_error",
            AppError::Api(_) => "api_error",
            AppError::Network(_) => "network_error",
            AppError::Io(_) => "io_error",
        }
    }

    fn detail(&self) -> &str {
        match self {
            AppError::Config(s)
            | AppError::Validation(s)
            | AppError::Api(s)
            | AppError::Network(s)
            | AppError::Io(s) => s,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::json!({
            "ok": false,
            "error": self.error_type(),
            "detail": self.detail(),
        })
        .to_string()
    }

    pub fn exit(&self) -> ! {
        eprintln!("{}", self.to_json());
        process::exit(self.exit_code());
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.detail())
    }
}

impl std::error::Error for AppError {}
