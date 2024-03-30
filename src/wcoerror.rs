use thiserror::Error;

#[derive(Error, Debug)]
pub enum WCOError {
    #[error("Invalid input: {0}")]
    ParsingError(String),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Network error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Plotting error: {0}")]
    PlottingError(String),
}
