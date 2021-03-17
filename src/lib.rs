mod aicensor;
mod client;
mod noproxy;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OkaeriSdkError {
    #[error("url '{url}' is invalid: {source:?}")]
    InvalidUrl {
        url: String,
        source: url::ParseError,
    },
    #[error("cannot parse '{from}' to int")]
    InvalidInt { from: String },
    #[error("{group}: {message}")]
    ResponseError { group: String, message: String },
    #[error("cannot parse to json: '{body}'")]
    ResponseParseError { body: String },
}
