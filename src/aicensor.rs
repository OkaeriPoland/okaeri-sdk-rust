use crate::OkaeriSdkError;
use crate::client::OkaeriClient;
use serde_json::json;
use std::env;
use std::time::Duration;
use url::Url;
use std::collections::HashMap;
use serde::{Deserialize};

type Result<T> = std::result::Result<T, OkaeriSdkError>;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct CensorPredictionInfoGeneral {
    pub swear: bool,
    pub breakdown: String,
    pub domains: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct CensorPredictionInfoDetails {
    pub basic_contains_hit: bool,
    pub exact_match_hit: bool,
    pub ai_label: String,
    pub ai_probability: f64,
    pub domains_list: Vec<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct CensorPredictionInfoElapsed {
    pub all: f64,
    pub processing: f64,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct CensorPredictionInfo {
    pub general: CensorPredictionInfoGeneral,
    pub details: CensorPredictionInfoDetails,
    pub elapsed: CensorPredictionInfoElapsed,
}

pub struct AiCensor {
    client: OkaeriClient
}

impl AiCensor {
    pub fn new<S: Into<String>>(token: S) -> Result<Self> {
        AiCensor::new_with_config(token, None, None)
    }

    pub fn new_with_config<S: Into<String>>(token: S, base_url: Option<&str>, timeout: Option<Duration>) -> Result<Self> {
        let base_url = OkaeriClient::read_base_url(
            base_url,
            "https://ai-censor.okaeri.eu",
            "OKAERI_SDK_AICENSOR_BASE_PATH")?;
        let timeout = OkaeriClient::read_timeout(
            timeout,
            Duration::from_secs(5),
            "OKAERI_SDK_TIMEOUT")?;
        let headers = OkaeriClient::singleton_map(
            String::from("Token"), token.into());
        let client = OkaeriClient::new(base_url, timeout, headers)?;
        Ok(AiCensor { client })
    }

    pub(crate) async fn get_prediction(self, phrase: &str) -> Result<CensorPredictionInfo> {
        let body = json!({"phrase": phrase.to_owned()});
        self.client.post::<CensorPredictionInfo>("/predict", &*body.to_string()).await
    }
}
