use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DnsKind {
    A,
    AAAA,
    UnSupported,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DnsRecord {
    A(std::net::Ipv4Addr),
    AAAA(std::net::Ipv6Addr),
    UnSupported,
}

impl From<&DnsRecord> for DnsKind {
    fn from(dr: &DnsRecord) -> Self {
        match dr {
            DnsRecord::A(_) => Self::A,
            DnsRecord::AAAA(_) => Self::AAAA,
            DnsRecord::UnSupported => Self::UnSupported,
        }
    }
}

#[derive(Debug, Error)]
pub enum RecoderManagerBuilderError {
    #[error("type is empty")]
    ErrTypeIsEmpty,
    #[error("unknown type: '{0}'")]
    ErrUnknownType(String),
}

#[async_trait]
pub trait RecoderManager {
    async fn get_records(&self, name: &str) -> Result<Vec<DnsRecord>, Box<dyn std::error::Error>>;
    async fn create_records(
        &self,
        name: &str,
        rcd: &DnsRecord,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn update_records(
        &self,
        name: &str,
        rcd: &DnsRecord,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct RecoderManagerBuilder {
    typ: String,
    cfg: HashMap<String, String>,
}

impl RecoderManagerBuilder {
    pub fn new() -> Self {
        Self {
            typ: String::new(),
            cfg: HashMap::new(),
        }
    }

    pub fn with_type(mut self, typ: impl AsRef<str>) -> Self {
        self.typ = String::from(typ.as_ref());
        self
    }

    pub fn with_cfg(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.cfg
            .insert(String::from(key.as_ref()), String::from(value.as_ref()));
        self
    }

    pub fn with_cfgs(mut self, cfgs: &HashMap<String, String>) -> Self {
        self.cfg
            .extend(cfgs.iter().map(|(a, b)| (a.clone(), b.clone())));
        self
    }

    pub fn build(self) -> Result<Box<dyn RecoderManager>, Box<dyn std::error::Error>> {
        match &self.typ[..] {
            "" => Err(Box::new(RecoderManagerBuilderError::ErrTypeIsEmpty)),
            #[cfg(feature = "cloudflare")]
            "cloudflare" => Ok(Box::new(crate::cloudflare::Cloudflare::new(&self.cfg)?)),
            _ => Err(Box::new(RecoderManagerBuilderError::ErrUnknownType(
                self.typ.clone(),
            ))),
        }
    }
}
