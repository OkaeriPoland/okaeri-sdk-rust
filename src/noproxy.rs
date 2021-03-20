use crate::client::OkaeriClient;
use crate::OkaeriSdkError;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use url::Url;

type Result<T> = std::result::Result<T, OkaeriSdkError>;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct NoProxyAddressInfoGeneral {
    pub ip: String,
    pub asn: u64,
    pub provider: String,
    pub country: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct NoProxyAddressInfoRisks {
    pub total: u64,
    pub proxy: bool,
    pub country: bool,
    pub asn: bool,
    pub provider: bool,
    pub abuser: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct NoProxyAddressInfoScore {
    pub noproxy: u64,
    pub abuseipdb: u64,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct NoProxyAddressInfoSuggestions {
    pub verify: bool,
    pub block: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct NoProxyAddressInfo {
    pub general: NoProxyAddressInfoGeneral,
    pub risks: NoProxyAddressInfoRisks,
    pub score: NoProxyAddressInfoScore,
    pub suggestions: NoProxyAddressInfoSuggestions,
}

pub struct NoProxy {
    client: OkaeriClient,
}

impl NoProxy {
    pub fn new<S: Into<String>>(token: S) -> Result<Self> {
        NoProxy::new_with_config(token, None, None)
    }

    pub fn new_with_config<S: Into<String>>(
        token: S,
        base_url: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Self> {
        let base_url = OkaeriClient::read_base_url(
            base_url,
            "https://noproxy-api.okaeri.eu",
            "OKAERI_SDK_NOPROXY_BASE_PATH",
        )?;
        let timeout =
            OkaeriClient::read_timeout(timeout, Duration::from_secs(5), "OKAERI_SDK_TIMEOUT")?;
        let headers = maplit::hashmap! { String::from("Authorization") => format!("Bearer {}", token.into()) };
        let client = OkaeriClient::new(base_url, timeout, headers)?;
        Ok(NoProxy { client })
    }

    pub(crate) async fn get_info(self, address: &str) -> Result<NoProxyAddressInfo> {
        self.client.get(format!("/v1/{}", address)).await
    }
}
