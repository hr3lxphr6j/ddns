use crate::dns::WhoAmi;
use anyhow::Result;
use async_trait::async_trait;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

const IPV4_ENDPOINT: &'static str = "https://icanhazip.com";
const IPV6_ENDPOINT: &'static str = "https://ipv6.icanhazip.com";

pub struct ICanHazIp {
    http_client: reqwest::Client,
}

impl ICanHazIp {
    pub fn new() -> Result<Self> {
        Ok(Self {
            http_client: reqwest::ClientBuilder::new().build()?,
        })
    }

    async fn do_get(&self, endpoint: &str) -> Result<String> {
        Ok(self
            .http_client
            .get(endpoint)
            .send()
            .await
            .map_err(|err| {
                log::error!("Failed to get ip from url: {}, err: {}", endpoint, err);
                err
            })?
            .text()
            .await?)
    }
}

#[async_trait]
impl WhoAmi for ICanHazIp {
    async fn fetch_ipv4_addr(&self) -> Result<Ipv4Addr> {
        Ok(Ipv4Addr::from_str(
            &self.do_get(IPV4_ENDPOINT).await?.trim(),
        )?)
    }

    async fn fetch_ipv6_addr(&self) -> Result<Ipv6Addr> {
        Ok(Ipv6Addr::from_str(
            &self.do_get(IPV6_ENDPOINT).await?.trim(),
        )?)
    }
}
