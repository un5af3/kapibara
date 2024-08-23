//! Show all option

use std::time::Duration;

use kapibara::{
    Codec, DispatchOption, DnsOption, InboundOption, OutboundOption, RouteOption, RouteRuleOption,
};
use kapibara_service::{
    socks::{option::SocksAuthOption, SocksInboundOption},
    vless::{option::VlessUserOption, VlessInboundOption, VlessOutboundOption},
    InboundServiceOption, OutboundServiceOption,
};
use kapibara_transport::{
    dns::option::{NameServerOption, Protocol, Strategy},
    option::{ClientOption, ServerOption},
    tcp::TcpServerOption,
    websocket::{WebSocketClientOption, WebSocketServerOption},
    ResolveOption, TlsCertOption, TlsClientOption, TlsServerOption, TransportClientOption,
    TransportServerOption,
};

fn main() {
    let option = DispatchOption {
        route: RouteOption {
            rules: vec![
                RouteRuleOption {
                    dns: false,
                    inbound: vec!["in-1".into()],
                    outbound: "out-1".into(),
                },
                RouteRuleOption {
                    dns: true,
                    inbound: vec!["in-2".into()],
                    outbound: "out-2".into(),
                },
            ],
        },
        dns: Some(DnsOption {
            resolve: ResolveOption {
                strategy: Strategy::Ipv4ThenIpv6,
                timeout: Duration::from_secs(5),
                servers: vec![NameServerOption {
                    protocol: Protocol::Udp,
                    address: "8.8.8.8:53".parse().unwrap(),
                }],
            },
        }),
        inbound: vec![
            InboundOption {
                tag: "in-1".into(),
                server: TransportServerOption {
                    opt: ServerOption::Tcp(TcpServerOption {
                        listen: "127.0.0.1:6868".parse().unwrap(),
                        tcp_nodelay: true,
                    }),
                    tls: None,
                },
                service: InboundServiceOption::Socks(SocksInboundOption {
                    auth: vec![SocksAuthOption::Username {
                        user: "test".into(),
                        pass: "test".into(),
                    }],
                }),
            },
            InboundOption {
                tag: "in-2".into(),
                server: TransportServerOption {
                    opt: ServerOption::Ws(WebSocketServerOption {
                        path: "/test".into(),
                        listen: "127.0.0.1:6868".parse().unwrap(),
                        tcp_nodelay: true,
                    }),
                    tls: Some(TlsServerOption {
                        alpn: vec!["http/1.1".into(), "http/2".into()],
                        certificate: TlsCertOption::File {
                            cert: "certs/test.crt".into(),
                            key: "certs/test.key".into(),
                        },
                    }),
                },
                service: InboundServiceOption::Vless(VlessInboundOption {
                    users: vec![VlessUserOption {
                        user: "test".into(),
                        uuid: "s17b1019d-a951-4bc5-a6e9-e8ece8aebcc3".to_string(),
                    }],
                }),
            },
        ],
        outbound: vec![
            OutboundOption {
                tag: "out-1".into(),
                timeout: Some(Duration::from_secs(30)),
                service: OutboundServiceOption::Vless(VlessOutboundOption {
                    uuid: "s17b1019d-a951-4bc5-a6e9-e8ece8aebcc3".to_string(),
                    flow: None,
                }),
                client: TransportClientOption {
                    opt: ClientOption::Ws(WebSocketClientOption {
                        addr: "test.com".into(),
                        port: 443,
                        path: "/test".to_string(),
                        tcp_nodelay: true,
                    }),
                    tls: Some(TlsClientOption {
                        insecure: false,
                        alpn: vec!["http/1.1".into(), "http/2".into()],
                        enable_sni: true,
                        server_name: "test.com".into(),
                    }),
                },
            },
            OutboundOption {
                tag: "out-2".into(),
                client: TransportClientOption {
                    opt: ClientOption::Empty,
                    tls: None,
                },
                service: OutboundServiceOption::Direct,
                timeout: Some(Duration::from_secs(30)),
            },
        ],
    };

    let yaml = Codec::Yaml.to_string(&option).unwrap();
    println!("Yaml Format:\n{}", yaml);

    let json = Codec::Json.to_string(&option).unwrap();
    println!("Json Format:\n{}", json);
}
