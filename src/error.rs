use chrono::Duration;
use thiserror::Error;

pub enum TestOutcome {
    Ok(Duration),
    SlowDown(Duration),
}

#[derive(Error, Debug)]
pub enum TestError {
    #[error("HTTP client error")]
    ReqwestError(#[from] reqwest::Error),
}
