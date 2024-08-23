//! Kapibara Inbound

use std::sync::Arc;

use crate::InboundError;
use kapibara_service::{InboundService, InboundServiceOption};
use kapibara_transport::{TransportServer, TransportServerOption};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundOption {
    pub tag: String,
    pub server: TransportServerOption,
    pub service: InboundServiceOption,
}

pub struct Inbound {
    tag: String,
    svc: Arc<InboundService>,
    srv: Arc<TransportServer>,
}

impl Inbound {
    pub fn init(in_opt: InboundOption) -> Result<Self, InboundError> {
        let svc = InboundService::init(in_opt.service)?;
        let srv = TransportServer::init(in_opt.server)?;

        Ok(Self {
            tag: in_opt.tag,
            svc: Arc::new(svc),
            srv: Arc::new(srv),
        })
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn get_tag(&self) -> String {
        self.tag.to_owned()
    }

    pub fn get_service(&self) -> Arc<InboundService> {
        self.svc.clone()
    }

    pub fn get_server(&self) -> Arc<TransportServer> {
        self.srv.clone()
    }
}
