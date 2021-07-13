#[cfg(feature = "cloudflare")]
mod cloudflare;

mod config;
mod dns;

use log;
use reqwest;
use std::error::Error;
use std::net::IpAddr;
use std::str::FromStr;

pub struct Application {
    config: config::AppConfig,
}

impl Application {
    pub fn new(file: impl AsRef<str>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config: config::AppConfig::new(file)?,
        })
    }

    async fn fetch_public_ip() -> Result<String, reqwest::Error> {
        Ok(reqwest::get("https://icanhazip.com").await?.text().await?)
    }

    pub async fn do_update(&self) -> Result<(), Box<dyn Error>> {
        let ip = Self::fetch_public_ip().await.map_err(|err| {
            log::error!("Failed to get ip: {}", err);
            err
        })?;
        // TODO(chigusa): support ipv6
        let ip = IpAddr::from_str(&ip.trim()).map_err(|err| {
            log::error!("Failed to parse ip[{}]: {}", ip, err);
            err
        })?;
        let dns_record = match ip {
            IpAddr::V4(addr) => dns::DnsRecord::A(addr),
            IpAddr::V6(addr) => dns::DnsRecord::AAAA(addr),
        };

        log::info!("IP: {}", ip);

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
                    if current_record == &dns_record {
                        log::info!("Record[{} => {:?}] is equal, skip", &domain, &dns_record);
                        continue;
                    }
                    rm.update_records(&domain, &dns_record)
                        .await
                        .map_err(|err| {
                            log::error!(
                                "Update record[{} => {:?}] failed, {}",
                                &domain,
                                &dns_record,
                                err
                            );
                            err
                        })?;
                    log::info!("Update record[{} => {:?}] successes", &domain, &dns_record)
                } else {
                    rm.create_records(&domain, &dns_record)
                        .await
                        .map_err(|err| {
                            log::error!(
                                "Create record[{} => {:?}] failed, {}",
                                &domain,
                                &dns_record,
                                err
                            );
                            err
                        })?;
                    log::info!("Create record[{} => {:?}] successes", &domain, &dns_record)
                }
            }
        }

        Ok(())
    }
}
