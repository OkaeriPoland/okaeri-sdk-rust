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
        let token = token.into();

        let base_url = match env::var("OKAERI_SDK_AICENSOR_BASE_PATH") {
            Ok(value) => value,
            Err(_) => String::from(base_url.unwrap_or("https://ai-censor.okaeri.eu"))
        };

        let base_url = Url::parse(base_url.as_str())
            .map_err(|source| OkaeriSdkError::InvalidUrl { url: base_url, source })?;

        let timeout = match env::var("OKAERI_SDK_TIMEOUT") {
            Ok(from) => {
                let value = from.parse::<u64>().map_err(|_| OkaeriSdkError::InvalidInt { from })?;
                Duration::from_millis(value)
            }
            Err(_) => timeout.unwrap_or(Duration::from_secs(5))
        };

        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert(String::from("Token"), token);

        let client = OkaeriClient::new(base_url, timeout, headers)?;
        Ok(AiCensor { client })
    }

    pub(crate) async fn get_prediction(self, phrase: &str) -> Result<CensorPredictionInfo> {
        let body = json!({
            "phrase": phrase.to_owned()
        });
        self.client.post::<CensorPredictionInfo>("/predict", &*body.to_string()).await
    }
}
