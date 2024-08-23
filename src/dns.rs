//! Kapibara Dns

use std::sync::Arc;

use kapibara_transport::{ResolveOption, Resolver};
use serde::{Deserialize, Serialize};

use crate::DnsError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsOption {
    #[serde(flatten)]
    pub resolve: ResolveOption,
}

pub struct Dns {
    resolver: Arc<Resolver>,
}

impl Dns {
    pub fn init(dns_opt: Option<DnsOption>) -> Result<Self, DnsError> {
        let resolver = if let Some(opt) = dns_opt {
            Resolver::new(opt.resolve)
        } else {
            Resolver::default()
        };

        Ok(Self {
            resolver: Arc::new(resolver),
        })
    }

    pub fn resolver(&self) -> &Resolver {
        &self.resolver
    }

    pub fn get_resolver(&self) -> Arc<Resolver> {
        self.resolver.clone()
    }
}
