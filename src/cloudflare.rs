use crate::dns::{DnsKind, DnsRecord, RecoderManager};
use async_trait::async_trait;
use cloudflare::endpoints::dns::{
    CreateDnsRecord, CreateDnsRecordParams, DnsContent, ListDnsRecords, ListDnsRecordsParams,
    UpdateDnsRecord, UpdateDnsRecordParams,
};
use cloudflare::framework::{
    async_api::ApiClient,
    auth::Credentials::UserAuthKey,
    response::{ApiResponse, ApiSuccess},
    Environment::Production,
    HttpApiClientConfig,
};
use std::collections::HashMap;
use std::error::Error;
use thiserror::Error;
use tokio::sync::Mutex;

pub const CFG_NAME_EMAIL: &'static str = "email";
pub const CFG_NAME_ZONE_ID: &'static str = "zoneId";
pub const CFG_NAME_KEY: &'static str = "key";

#[derive(Debug, Error)]
pub enum CloudflareError {
    #[error("email is empty")]
    ErrEmailIsEmpty,
    #[error("zone id is empty")]
    ErrZoneIdIsEmpty,
    #[error("key is empty")]
    ErrKeyIsEmpty,
    #[error("record[{record_name}] not exist")]
    ErrRecordNotExist { record_name: String },
}

impl From<&DnsContent> for DnsRecord {
    fn from(dc: &DnsContent) -> Self {
        match dc {
            DnsContent::A { content } => Self::A(content.clone()),
            DnsContent::AAAA { content } => Self::AAAA(content.clone()),
            _ => Self::UnSupported,
        }
    }
}

impl From<&DnsRecord> for DnsContent {
    fn from(dr: &DnsRecord) -> Self {
        match dr {
            DnsRecord::A(content) => DnsContent::A {
                content: content.clone(),
            },
            DnsRecord::AAAA(content) => DnsContent::AAAA {
                content: content.clone(),
            },
            DnsRecord::UnSupported => panic!("Unable to convert unsupported DNS records."),
        }
    }
}

impl From<&DnsContent> for DnsKind {
    fn from(dc: &DnsContent) -> Self {
        match dc {
            DnsContent::A { .. } => Self::A,
            DnsContent::AAAA { .. } => Self::AAAA,
            _ => Self::UnSupported,
        }
    }
}

pub struct Cloudflare {
    zone_id: String,
    client: Mutex<cloudflare::framework::async_api::Client>,
}

impl Cloudflare {
    pub fn new(cfg: &HashMap<String, String>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut email = String::new();
        let mut zone_id = String::new();
        let mut key = String::new();

        for i in [
            (&mut email, CFG_NAME_EMAIL, CloudflareError::ErrEmailIsEmpty),
            (
                &mut zone_id,
                CFG_NAME_ZONE_ID,
                CloudflareError::ErrZoneIdIsEmpty,
            ),
            (&mut key, CFG_NAME_KEY, CloudflareError::ErrKeyIsEmpty),
        ] {
            if let Some(v) = cfg.get(i.1) {
                *i.0 = v.clone()
            } else {
                return Err(Box::new(i.2));
            }
        }

        let auth = UserAuthKey { email, key };
        let cfg: HttpApiClientConfig = Default::default();
        let client = Mutex::new(cloudflare::framework::async_api::Client::new(
            auth, cfg, Production,
        )?);
        Ok(Self { zone_id, client })
    }

    async fn get_raw_records(
        &self,
        name: &str,
    ) -> ApiResponse<Vec<cloudflare::endpoints::dns::DnsRecord>> {
        self.client
            .lock()
            .await
            .request(&ListDnsRecords {
                zone_identifier: &self.zone_id,
                params: ListDnsRecordsParams {
                    record_type: None,
                    name: Some(String::from(name)),
                    page: None,
                    per_page: None,
                    order: None,
                    direction: None,
                    search_match: None,
                },
            })
            .await
    }
}

#[async_trait]
impl RecoderManager for Cloudflare {
    async fn get_records(&self, name: &str) -> Result<Vec<DnsRecord>, Box<dyn std::error::Error>> {
        let resp: ApiSuccess<Vec<cloudflare::endpoints::dns::DnsRecord>> =
            self.get_raw_records(name).await?;
        Ok(resp
            .result
            .iter()
            .filter_map(|r| Some(DnsRecord::from(&r.content)))
            .collect())
    }

    async fn create_records(&self, name: &str, rcd: &DnsRecord) -> Result<(), Box<dyn Error>> {
        let endpoint = CreateDnsRecord {
            zone_identifier: &self.zone_id,
            params: CreateDnsRecordParams {
                ttl: None,
                priority: None,
                proxied: Some(false),
                name,
                content: rcd.into(),
            },
        };
        self.client.lock().await.request(&endpoint).await?;
        Ok(())
    }

    async fn update_records(&self, name: &str, rcd: &DnsRecord) -> Result<(), Box<dyn Error>> {
        let resp = self.get_raw_records(name).await?;

        let identifier;

        if let Some(v) = resp
            .result
            .into_iter()
            .find(|dr| &dr.name == name && DnsKind::from(&dr.content) == DnsKind::from(rcd))
        {
            identifier = v.id
        } else {
            return Err(Box::new(CloudflareError::ErrRecordNotExist {
                record_name: String::from(name),
            }));
        };

        let endpoint = UpdateDnsRecord {
            zone_identifier: &self.zone_id,
            identifier: &identifier,
            params: UpdateDnsRecordParams {
                ttl: Some(1),
                proxied: Some(false),
                name,
                content: rcd.into(),
            },
        };
        self.client.lock().await.request(&endpoint).await?;
        Ok(())
    }
}
