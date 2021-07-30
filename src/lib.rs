#[cfg(feature = "cloudflare")]
mod cloudflare;

mod config;
mod dns;

use crate::dns::DnsRecord;
use anyhow::Result;
use log;
use reqwest;
use std::net::IpAddr;
use std::str::FromStr;

const IPV4_ENDPOINT: &'static str = "https://icanhazip.com";
#[cfg(feature = "ipv6")]
const IPV6_ENDPOINT: &'static str = "https://ipv6.icanhazip.com";

pub struct Application {
    config: config::AppConfig,
    http_client: reqwest::Client,
}

impl Application {
    pub fn new(file: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            config: config::AppConfig::new(file)?,
            http_client: reqwest::ClientBuilder::new().build()?,
        })
    }

    async fn fetch_public_ip(&self, endpoint: impl AsRef<str>) -> Result<IpAddr> {
        let content = self
            .http_client
            .get(endpoint.as_ref())
            .send()
            .await
            .map_err(|err| {
                log::error!(
                    "Failed to get ip from url: {}, err: {}",
                    endpoint.as_ref(),
                    err
                );
                err
            })?
            .text()
            .await?;
        Ok(IpAddr::from_str(content.trim()).map_err(|err| {
            log::error!("Failed to parse ip from: {}, err: {}", content.trim(), err);
            err
        })?)
    }

    async fn _do_update(&self, dns_record: &DnsRecord) -> Result<()> {
        for item in &self.config.services {
            let rm = dns::RecoderManagerBuilder::new()
                .with_type(&item.name)
                .with_cfgs(&item.config)
                .build()
                .map_err(|err| {
                    log::error!("Failed to init recoder manager: {}", err);
                    err
                })?;
            for domain in &item.domain {
                let res = rm.get_records(&domain).await.map_err(|err| {
                    log::error!("Failed to get record, domain[{}], {}", domain, err);
                    err
                })?;
                if res.len() > 0 {
                    let current_record = res.get(0).unwrap();
                    log::info!("Current record[{} => {:?}]", domain, current_record);
                    if current_record == dns_record {
                        log::info!("Record[{} => {:?}] is equal, skip", &domain, dns_record);
                        continue;
                    }
                    rm.update_records(&domain, dns_record)
                        .await
                        .map_err(|err| {
                            log::error!(
                                "Update record[{} => {:?}] failed, {}",
                                &domain,
                                dns_record,
                                err
                            );
                            err
                        })?;
                    log::info!("Update record[{} => {:?}] successes", &domain, dns_record)
                } else {
                    rm.create_records(&domain, dns_record)
                        .await
                        .map_err(|err| {
                            log::error!(
                                "Create record[{} => {:?}] failed, {}",
                                &domain,
                                dns_record,
                                err
                            );
                            err
                        })?;
                    log::info!("Create record[{} => {:?}] successes", &domain, dns_record)
                }
            }
        }
        Ok(())
    }

    pub async fn do_update(&self) -> Result<()> {
        for endpoint in [
            IPV4_ENDPOINT,
            #[cfg(feature = "ipv6")]
            IPV6_ENDPOINT,
        ] {
            let ip = self.fetch_public_ip(endpoint).await;
            if let Err(..) = ip {
                log::error!("failed to get ip from: {}, skip", endpoint);
                continue;
            }
            let ip = ip.unwrap();

            let dns_record = match ip {
                IpAddr::V4(addr) => dns::DnsRecord::A(addr),
                IpAddr::V6(addr) => dns::DnsRecord::AAAA(addr),
            };

            log::info!("IP: {}", ip);

            self._do_update(&dns_record).await?;
        }

        Ok(())
    }
}
