/*
 * Okaeri SDK (Rust)
 * Copyright (C) 2021 Okaeri, Dawid Sawicki
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use crate::OkaeriSdkError;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Method, Request};
use hyper_timeout::TimeoutConnector;
use hyper_tls::HttpsConnector;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use url::Url;

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
        Ok(OkaeriClient {
            base_url,
            hyper,
            headers,
        })
    }

    pub(crate) fn read_base_url(provided: Option<&str>, def: &str, env_name: &str) -> Result<Url> {
        let base_url = match env::var(env_name) {
            Ok(value) => value,
            Err(_) => String::from(provided.unwrap_or(def)),
        };
        let base_url =
            Url::parse(base_url.as_str()).map_err(|source| OkaeriSdkError::InvalidUrl {
                url: base_url,
                source,
            })?;
        Ok(base_url)
    }

    pub(crate) fn read_timeout(
        provided: Option<Duration>,
        def: Duration,
        env_name: &str,
    ) -> Result<Duration> {
        let timeout = match env::var(env_name) {
            Ok(from) => {
                let value = from
                    .parse::<u64>()
                    .map_err(|_| OkaeriSdkError::InvalidInt { from })?;
                Duration::from_millis(value)
            }
            Err(_) => provided.unwrap_or(def),
        };
        Ok(timeout)
    }

    pub(crate) async fn post<T>(self, path: impl AsRef<str>, body: impl Into<String>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request(path, body, Method::POST).await
    }

    pub(crate) async fn get<T>(self, path: impl AsRef<str>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request(path, "", Method::GET).await
    }

    async fn request<T>(
        self,
        path: impl AsRef<str>,
        body: impl Into<String>,
        method: impl Into<Method>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let path = path.as_ref();
        let body = body.into();
        let method = method.into();

        let url = format!("{}{}", self.base_url, path);
        let mut req = Request::builder().method(method).uri(url);

        for (key, value) in self.headers {
            req = req.header(key.as_str(), value.as_str());
        }

        let req = req
            .body(Body::from(body))
            .map_err(|err| OkaeriSdkError::ResponseError {
                group: String::from("REQUEST_ERROR"),
                message: format!("failed to create request: {}", err),
            })?;

        let res = self
            .hyper
            .request(req)
            .await
            .map_err(|err| OkaeriSdkError::ResponseError {
                group: String::from("REQUEST_ERROR"),
                message: format!("failed to dispatch request: {}", err),
            })?;

        if !res.status().is_success() {
            let error = OkaeriSdkError::ResponseError {
                group: String::from("REQUEST_ERROR"),
                message: format!("received invalid status code {}", res.status()),
            };
            return Err(error);
        }

        let bytes =
            hyper::body::to_bytes(res)
                .await
                .map_err(|err| OkaeriSdkError::ResponseError {
                    group: String::from("REQUEST_ERROR"),
                    message: format!("failed to process request: {}", err),
                })?;

        let body_str =
            String::from_utf8(bytes.to_vec()).map_err(|err| OkaeriSdkError::ResponseError {
                group: String::from("REQUEST_ERROR"),
                message: format!("failed to convert body to string: {}", err),
            })?;

        let info: T =
            serde_json::from_str(&body_str).map_err(|_| OkaeriSdkError::ResponseParseError {
                body: body_str.clone(),
            })?;

        Ok(info)
    }
}
