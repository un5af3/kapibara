//! Kapibara Outbound

use std::{sync::Arc, time::Duration};

use kapibara_service::{OutboundService, OutboundServiceOption};
use kapibara_transport::{Resolver, TransportClient, TransportClientOption};
use serde::{Deserialize, Serialize};

use crate::OutboundError;

fn default_timeout() -> Option<Duration> {
    Some(Duration::from_secs(30))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundOption {
    pub tag: String,
    // default is empty client, return an empty stream
    #[serde(default)]
    pub client: TransportClientOption,
    pub service: OutboundServiceOption,
    // connect timeout, default 30s
    #[serde(default = "default_timeout")]
    pub timeout: Option<Duration>,
}

pub struct Outbound {
    tag: String,
    svc: Arc<OutboundService>,
    cli: Arc<TransportClient>,
    timeout: Option<Duration>,
}

impl Outbound {
    pub fn init(out_opt: OutboundOption, resolver: &Resolver) -> Result<Self, OutboundError> {
        let cli = TransportClient::init(out_opt.client, resolver)?;
        let svc = OutboundService::init(out_opt.service)?;

        Ok(Self {
            tag: out_opt.tag,
            svc: Arc::new(svc),
            cli: Arc::new(cli),
            timeout: out_opt.timeout,
        })
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn get_tag(&self) -> String {
        self.tag.to_owned()
    }

    pub fn get_service(&self) -> Arc<OutboundService> {
        self.svc.clone()
    }

    pub fn get_client(&self) -> Arc<TransportClient> {
        self.cli.clone()
    }

    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }
}
