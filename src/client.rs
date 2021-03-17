use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use hyper_timeout::TimeoutConnector;
use hyper::{Client, Method, Request, Body};
use crate::OkaeriSdkError;
use std::time::Duration;
use url::Url;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, OkaeriSdkError>;

pub(crate) struct OkaeriClient {
    base_url: Url,
    hyper: Client<TimeoutConnector<HttpsConnector<HttpConnector>>>,
    headers: HashMap<String, String>,
}

impl OkaeriClient {
    pub fn new(base_url: Url, timeout: Duration, headers: HashMap<String, String>) -> Result<Self> {
        let https = HttpsConnector::new();
        let mut connector = TimeoutConnector::new(https);
        connector.set_connect_timeout(Some(timeout));
        connector.set_read_timeout(Some(timeout));
        connector.set_write_timeout(Some(timeout));
        let hyper = Client::builder().build::<_, hyper::Body>(connector);
        Ok(OkaeriClient { base_url, hyper, headers })
    }

    pub(crate) async fn post<T>(self, path: &str, body: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);

        let mut req = Request::builder()
            .method(Method::POST)
            .uri(url);

        for (key, value) in self.headers {
            req = req.header(key.as_str(), value.as_str());
        }

        let req = req.body(Body::from(body))
            .map_err(|err| OkaeriSdkError::ResponseError {
                group: String::from("REQUEST_ERROR"),
                message: String::from(format!("failed to create request: {}", err)),
            })?;

        let res = self.hyper.request(req).await
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

        let info: T = serde_json::from_reader(&*body_str)
            .map_err(|_| OkaeriSdkError::ResponseParseError { body: body_str })?;

        Ok(info)
    }
}
