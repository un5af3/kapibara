## Kapibara

Kapibara is a proxy platform write by Rust.

It can construct with any transport (websocket, tcp, ...) with any protocol (vless, socks, ...).

## Example

### Client Config

```
route:
  rules:
    - dns: false
      inbound:
        - in-1
      outbound: out-1
inbound:
  - tag: in-1
    server:
      opt: !tcp
        listen: 127.0.0.1:6868
        tcp_nodelay: true
    service: !socks
outbound:
  - tag: out-1
    client:
      opt: !ws
        addr: <address>
        port: 8686
        path: /test
        tcp_nodelay: true
      tls:
        insecure: true
        enable_sni: false
    service: !vless
      uuid: b1eb0b94-8f57-438b-97d5-e79090cc5108
```

### Server Config

```
dns:
  strategy: ipv4_then_ipv6
  timeout:
    secs: 5
    nanos: 0
  servers:
    - protocol: udp
      address: 8.8.8.8:53
route:
  rules:
    - dns: true
      inbound:
        - in-2
      outbound: out-2
inbound:
  - tag: in-2
    server:
      opt: !ws
        listen: 0.0.0.0:8686
        path: /test
        tcp_nodelay: true
      tls:
        certificate: !file
          cert: certs/test.crt
          key: certs/test.key
    service: !vless
      users:
        - user: test
          uuid: b1eb0b94-8f57-438b-97d5-e79090cc5108
outbound:
  - tag: out-2
    service: direct
    timeout:
      secs: 30
      nanos: 0

```
