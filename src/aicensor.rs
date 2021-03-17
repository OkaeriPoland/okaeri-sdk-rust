use crate::OkaeriSdkError;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde_json::json;
use std::env;
use std::time::Duration;
use url::Url;
use hyper_timeout::TimeoutConnector;

type Result<T> = std::result::Result<T, OkaeriSdkError>;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct CensorPredictionInfoGeneral {
    pub(crate) swear: bool,
    pub(crate) breakdown: String,
    pub(crate) domains: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct CensorPredictionInfoDetails {
    pub(crate) basic_contains_hit: bool,
    pub(crate) exact_match_hit: bool,
    pub(crate) ai_label: String,
    pub(crate) ai_probability: f64,
    pub(crate) domains_list: Vec<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct CensorPredictionInfoElapsed {
    pub(crate) all: f64,
    pub(crate) processing: f64,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct CensorPredictionInfo {
    pub(crate) general: CensorPredictionInfoGeneral,
    pub(crate) details: CensorPredictionInfoDetails,
    pub(crate) elapsed: CensorPredictionInfoElapsed,
}

pub struct AiCensor {
    base_url: Url,
    token: String,
    timeout: Duration,
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

        Ok(AiCensor { base_url, timeout, token })
    }

    pub(crate) async fn get_prediction(self, phrase: &str) -> Result<CensorPredictionInfo> {
        let url = format!("{}/predict", self.base_url);
        let body = json!({
            "phrase": phrase.to_owned()
        });

        let https = HttpsConnector::new();
        let mut connector = TimeoutConnector::new(https);
        connector.set_connect_timeout(Some(self.timeout));
        connector.set_read_timeout(Some(self.timeout));
        connector.set_write_timeout(Some(self.timeout));
        let client = Client::builder().build::<_, hyper::Body>(connector);

        let req = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("Token", self.token)
            .body(Body::from(body.to_string()))
            .map_err(|err| OkaeriSdkError::ResponseError {
                group: String::from("REQUEST_ERROR"),
                message: String::from(format!("failed to create request: {}", err)),
            })?;

        let res = client.request(req).await
            .map_err(|err| OkaeriSdkError::ResponseError {
                group: String::from("REQUEST_ERROR"),
                message: String::from(format!("failed to dispatch request: {}", err)),
            })?;

        if !res.status().is_success() {
            let error = OkaeriSdkError::ResponseError {
                group: String::from("REQUEST_ERROR"),
                message: String::from(format!("received invalid status code {}", res.status())),
            };
            return Err(error);
        }

        let bytes = hyper::body::to_bytes(res).await
            .map_err(|err| OkaeriSdkError::ResponseError {
                group: String::from("REQUEST_ERROR"),
                message: String::from(format!("failed to process request: {}", err)),
            })?;

        let body_str = String::from_utf8(bytes.to_vec())
            .map_err(|err| OkaeriSdkError::ResponseError {
                group: String::from("REQUEST_ERROR"),
                message: String::from(format!("failed to convert body to string: {}", err)),
            })?;

        let info: CensorPredictionInfo = serde_json::from_str(&*body_str)
            .map_err(|_| OkaeriSdkError::ResponseParseError { body: body_str })?;

        Ok(info)
    }
}
