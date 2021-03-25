# Okaeri SDK for Rust

![License](https://img.shields.io/github/license/OkaeriPoland/okaeri-sdk-java)
![Total lines](https://img.shields.io/tokei/lines/github/OkaeriPoland/okaeri-sdk-java)
![Repo size](https://img.shields.io/github/repo-size/OkaeriPoland/okaeri-sdk-java)
![crates.io](https://img.shields.io/crates/v/okaeri-sdk)
![Contributors](https://img.shields.io/github/contributors/OkaeriPoland/okaeri-sdk-java)
[![Discord](https://img.shields.io/discord/589089838200913930)](https://discord.gg/hASN5eX)

Currently supported services:
- [OK! AI.Censor](#ok-aicensor)
- [OK! No.Proxy](#ok-noproxy)

Full documentation available on [wiki.okaeri.eu](https://wiki.okaeri.eu/) in:
- [Polish](https://wiki.okaeri.eu/pl/sdk/rust)
- [English](https://wiki.okaeri.eu/en/sdk/rust)

## Cargo.toml definition
```
okaeri-sdk = "1.*"
```

## Example usage
### OK! AI.Censor
See full docs in: [Polish](https://wiki.okaeri.eu/pl/sdk/rust#ok-aicensor), [English](https://wiki.okaeri.eu/en/sdk/rust#ok-aicensor)
```rust
use okaeri_sdk::aicensor::AiCensor;

#[tokio::main]
async fn main() {
    let aicensor = AiCensor::new("XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX")?;
    let prediction = aicensor.get_prediction("o cie k u r//w@!").await?;
    let swear = prediction.general.swear;
    println!("swear: {}", swear);
}
```

### OK! No.Proxy
See full docs in: [Polish](https://wiki.okaeri.eu/pl/sdk/rust#ok-noproxy), [English](https://wiki.okaeri.eu/en/sdk/rust#ok-noproxy)
```rust
use okaeri_sdk::noproxy::NoProxy;

#[tokio::main]
async fn main() {
    let noproxy = NoProxy::new("XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX")?;
    let info = noproxy.get_info("1.1.1.1").await?;
    let proxy = info.risks.proxy;
    let verify = info.suggestions.verify;
    let block = info.suggestions.block;
    println!("proxy: {}, verify: {}, block: {}", proxy, verify, block);
}
```
