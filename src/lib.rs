#[cfg(feature = "cloudflare")]
mod cloudflare;

mod config;
mod dns;
mod icanhazip;

use crate::dns::{DnsRecord, WhoAmi};
use crate::icanhazip::ICanHazIp;
use anyhow::Result;
use log;

pub struct Application {
    config: config::AppConfig,
    who_am_i: Box<dyn WhoAmi>,
}

impl Application {
    pub fn new(file: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            config: config::AppConfig::new(file)?,
            who_am_i: Box::new(ICanHazIp::new()?),
        })
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
        self._do_update(&DnsRecord::A(self.who_am_i.fetch_ipv4_addr().await?))
            .await?;
        if self.config.ipv6 {
            self._do_update(&DnsRecord::AAAA(self.who_am_i.fetch_ipv6_addr().await?))
                .await?;
        }
        Ok(())
    }
}
