//! Kapibara Error Handle
use kapibara_service::{InboundError as InServiceError, OutboundError as OutServiceError};
use kapibara_transport::{ClientError, ResolveError, ServerError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DispatchError {
    #[error("[dns] {0}")]
    Dns(#[from] DnsError),
    #[error("[inbound] {0}")]
    Inbound(#[from] InboundError),
    #[error("[outbound] {0}")]
    Outbound(#[from] OutboundError),
    #[error("[route] {0}")]
    Route(#[from] RouteError),
    #[error("[option] {0}")]
    Option(#[from] OptionError),
}

#[derive(Debug, Error)]
pub enum InboundError {
    #[error("<server> {0}")]
    Server(#[from] ServerError),
    #[error("<service> {0}")]
    Service(#[from] InServiceError),
    #[error("<option> {0}")]
    Option(#[from] OptionError),
}

#[derive(Debug, Error)]
pub enum OutboundError {
    #[error("<client> {0}")]
    Client(#[from] ClientError),
    #[error("<service> {0}")]
    Service(#[from] OutServiceError),
    #[error("<option> {0}")]
    Option(#[from] OptionError),
}

#[derive(Debug, Error)]
pub enum RouteError {
    #[error("<option> {0}")]
    Option(#[from] OptionError),
}

#[derive(Debug, Error)]
pub enum OptionError {
    #[error("unknown tag ({0})")]
    UnknownTag(String),
    #[error("duplicate tag ({0})")]
    DuplicateTag(String),
    #[error("serialize ({0})")]
    Serialize(String),
    #[error("deserialize ({0})")]
    Deserialize(String),
}

#[derive(Debug, Error)]
pub enum DnsError {
    #[error("<resolve> {0}")]
    Resolve(#[from] ResolveError),
    #[error("<option> {0}")]
    Option(#[from] OptionError),
    #[error("<init> {0}")]
    Init(String),
}
