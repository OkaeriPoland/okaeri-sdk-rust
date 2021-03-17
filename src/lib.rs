mod aicensor;

#[macro_use]
extern crate serde_derive;

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

#[cfg(test)]
mod tests {
    use crate::aicensor::AiCensor;

    type TestResult = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

    #[tokio::test]
    async fn aicensor() -> TestResult {

        let aicensor = AiCensor::new("94433c4b-6ae2-479e-a051-731a2ef1919a")?;
        let prediction = aicensor.get_prediction("hehe").await?;
        let swear = prediction.general.swear;
        println!("swear: {}", swear);

        Ok(())
    }
}
