//! Kapibara Dispatch

use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use kapibara_service::{
    Address, InboundService, InboundServiceTrait, OutboundPacket, OutboundService,
    OutboundServiceTrait, ServiceAddress,
};
use kapibara_transport::{
    Resolver, TransportClient, TransportClientTrait, TransportServerCallback, TransportServerTrait,
};
use serde::{Deserialize, Serialize};
use tokio::{io::BufStream, task::JoinHandle};

use crate::{
    dns::Dns,
    error::OptionError,
    io::{copy_bi, ToStreamTimer},
    DispatchError, DnsOption, Inbound, InboundOption, Outbound, OutboundOption, Route, RouteOption,
};

const SERVER_RETRY: u8 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchOption {
    pub dns: Option<DnsOption>,
    pub route: RouteOption,
    pub inbound: Vec<InboundOption>,
    pub outbound: Vec<OutboundOption>,
}

pub struct Dispatch {
    dns: Dns,
    route: Route,
    inbound: HashMap<String, Inbound>,
    outbound: HashMap<String, Outbound>,

    in_state: HashMap<String, Option<JoinHandle<()>>>,
}

impl Dispatch {
    pub fn init(option: DispatchOption) -> Result<Self, DispatchError> {
        let dns = Dns::init(option.dns)?;

        let route = Route::init(option.route)?;

        let mut inbound = HashMap::new();
        for in_opt in option.inbound {
            let i = Inbound::init(in_opt)?;
            if let Some(other) = inbound.insert(i.tag().to_owned(), i) {
                return Err(DispatchError::Option(OptionError::DuplicateTag(
                    other.tag().to_owned(),
                )));
            }
        }

        let mut outbound = HashMap::new();
        for out_opt in option.outbound {
            let o = Outbound::init(out_opt, dns.resolver())?;
            if let Some(other) = outbound.insert(o.tag().to_owned(), o) {
                return Err(DispatchError::Option(OptionError::DuplicateTag(
                    other.tag().to_owned(),
                )));
            }
        }

        Ok(Self {
            dns,
            route,
            inbound,
            outbound,

            in_state: HashMap::new(),
        })
    }

    pub fn start(&mut self) -> Result<(), DispatchError> {
        for (in_tag, rule) in self.route.in_to_out.iter() {
            let inbound =
                self.inbound
                    .get(in_tag)
                    .ok_or(DispatchError::Option(OptionError::UnknownTag(
                        in_tag.to_owned(),
                    )))?;

            let outbound = self
                .outbound
                .get(&rule.outbound)
                .ok_or(DispatchError::Option(OptionError::UnknownTag(
                    rule.outbound.to_owned(),
                )))?;

            let resolver = if rule.dns {
                Some(self.dns.get_resolver())
            } else {
                None
            };

            let server = inbound.get_server();

            log::info!(
                "[inbound] start {} server {}",
                server.name(),
                if let Some(addr) = server.local_addr() {
                    addr.to_string()
                } else {
                    "".to_string()
                }
            );

            let callback = DispatchCallback::new(inbound, outbound, resolver);
            let task = tokio::spawn(async move {
                for i in 0..SERVER_RETRY {
                    if let Err(e) = server.serve(callback.clone()).await {
                        if i < SERVER_RETRY - 1 {
                            log::error!("[inbound] <server> {}", e);
                        } else {
                            panic!("[inbound] <server> {}", e);
                        }
                    }
                }
            });

            if let Some(_) = self.in_state.insert(in_tag.to_owned(), Some(task)) {
                return Err(DispatchError::Option(OptionError::DuplicateTag(
                    in_tag.to_owned(),
                )));
            }
        }

        Ok(())
    }

    pub fn close(&mut self) {
        for state in self.in_state.iter_mut() {
            if let Some(h) = state.1.take() {
                log::info!("[inbound]({}) closed", state.0);
                h.abort();
            }
        }
    }
}

#[derive(Clone)]
pub struct DispatchCallback {
    resolver: Option<Arc<Resolver>>,

    in_tag: String,
    in_svc: Arc<InboundService>,

    out_tag: String,
    out_svc: Arc<OutboundService>,
    out_cli: Arc<TransportClient>,
    timeout: Option<Duration>,
}

impl DispatchCallback {
    pub fn new(inbound: &Inbound, outbound: &Outbound, resolver: Option<Arc<Resolver>>) -> Self {
        Self {
            resolver,
            in_tag: inbound.get_tag(),
            in_svc: inbound.get_service(),
            out_tag: outbound.get_tag(),
            out_svc: outbound.get_service(),
            out_cli: outbound.get_client(),
            timeout: outbound.timeout(),
        }
    }
}

impl TransportServerCallback for DispatchCallback {
    async fn handle<S>(&self, stream: S, addr: Option<SocketAddr>)
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + Sync,
    {
        let stream = BufStream::new(stream);
        let (mut in_stream, in_pac) = match self.in_svc.handshake(stream).await {
            Ok((s, p)) => (s, p),
            Err(e) => {
                log::error!("[inbound] {}", e);
                return;
            }
        };

        log::info!(
            "[dispatch] [{}({}) -> {}({})] (<{}>{}) {}://{}",
            self.in_svc.name(),
            self.in_tag,
            self.out_svc.name(),
            self.out_tag,
            in_pac.detail,
            if let Some(a) = addr {
                a.to_string()
            } else {
                String::new()
            },
            in_pac.typ,
            in_pac.dest
        );

        let dest = if let Some(ref resolver) = self.resolver {
            match in_pac.dest.addr {
                Address::Domain(domain) => {
                    let mut resolved = match resolver.resolve(&domain, in_pac.dest.port).await {
                        Ok(r) => r,
                        Err(e) => {
                            log::error!("[dns] <resolve> {}", e);
                            return;
                        }
                    };

                    let addr = match resolved.next() {
                        Some(a) => a,
                        None => {
                            log::error!("[dns] <resolve> empty resolved");
                            return;
                        }
                    };

                    ServiceAddress::new(Address::Socket(addr.ip()), addr.port())
                }
                Address::Socket(_) => in_pac.dest,
            }
        } else {
            in_pac.dest
        };

        let out_pac = OutboundPacket {
            typ: in_pac.typ,
            dest,
        };

        let cli_stream = match self.out_cli.connect().await {
            Ok(s) => s,
            Err(e) => {
                log::debug!("[outbound] <client> {}", e);
                return;
            }
        };

        // if cli_stream is empty, so the timer need to set after handshake
        // else cli_stream need to set timer first, because handshake need.
        if cli_stream.is_emtpy() {
            let out_stream = match self.out_svc.handshake(cli_stream, out_pac).await {
                Ok(s) => s.to_timer(self.timeout),
                Err(e) => {
                    log::debug!("[outbound] {}", e);
                    return;
                }
            };

            let mut out_stream = BufStream::new(out_stream);

            let (_tx, _rx) = match copy_bi(&mut in_stream, &mut out_stream).await {
                Ok(s) => s,
                Err(e) => {
                    log::debug!("[transport] {}", e);
                    return;
                }
            };
        } else {
            let cli_stream = BufStream::new(cli_stream.to_timer(self.timeout));

            let mut out_stream = match self.out_svc.handshake(cli_stream, out_pac).await {
                Ok(s) => s,
                Err(e) => {
                    log::debug!("[outbound] {}", e);
                    return;
                }
            };

            let (_tx, _rx) = match copy_bi(&mut in_stream, &mut out_stream).await {
                Ok(s) => s,
                Err(e) => {
                    log::debug!("[transport] {}", e);
                    return;
                }
            };
        }
    }
}
