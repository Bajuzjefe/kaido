use thiserror::Error;

#[derive(Error, Debug)]
pub enum KaidoError {
    #[error("Template rendering failed: {0}")]
    TemplateError(#[from] tera::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid option: {0}")]
    InvalidOption(String),

    #[error("Aiken build failed:\n{0}")]
    AikenBuildFailed(String),

    #[error("Aiken check failed:\n{0}")]
    AikenCheckFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Aikido scan found issues:\n{0}")]
    AikidoScanFailed(String),
}

pub type Result<T> = std::result::Result<T, KaidoError>;
