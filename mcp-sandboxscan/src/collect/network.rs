use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use serde::{Deserialize, Serialize};
use wasmtime_wasi::sockets::SocketAddrUse;

use crate::monitor::event::{MonitorEvent, MonitorEventKind};
use crate::taint::source::TaintSource;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkDirection {
    Outbound,
    Inbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkObservation {
    pub direction: NetworkDirection,
    pub protocol: String,
    pub remote_host: String,
    pub remote_port: Option<u16>,
    pub url: Option<String>,
    pub allowed: bool,
    pub bytes_sent: Option<usize>,
}

#[derive(Default)]
pub struct NetworkCollector {
    observations: Mutex<Vec<NetworkObservation>>,
    proxy_handle: Mutex<Option<JoinHandle<()>>>,
}

impl NetworkCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, obs: NetworkObservation) {
        self.observations.lock().unwrap().push(obs);
    }

    pub fn take(&self) -> Vec<NetworkObservation> {
        std::mem::take(&mut *self.observations.lock().unwrap())
    }

    pub fn observations(&self) -> Vec<NetworkObservation> {
        self.observations.lock().unwrap().clone()
    }

    pub fn as_taint_sources(&self) -> Vec<TaintSource> {
        self.observations()
            .into_iter()
            .map(|obs| {
                let endpoint = match obs.remote_port {
                    Some(port) => format!("{}:{}", obs.remote_host, port),
                    None => obs.remote_host.clone(),
                };
                let content = obs
                    .url
                    .clone()
                    .unwrap_or_else(|| format!("{}://{}", obs.protocol, endpoint));
                TaintSource::NetworkConnect {
                    host: obs.remote_host,
                    port: obs.remote_port.unwrap_or(0),
                    protocol: obs.protocol,
                    content,
                }
            })
            .collect()
    }

    pub fn as_monitor_events(&self) -> Vec<MonitorEvent> {
        self.observations()
            .into_iter()
            .map(|obs| {
                let target = obs
                    .url
                    .clone()
                    .or_else(|| {
                        obs.remote_port
                            .map(|port| format!("{}:{}", obs.remote_host, port))
                    })
                    .or(Some(obs.remote_host.clone()));

                MonitorEvent {
                    kind: if obs.allowed {
                        MonitorEventKind::NetworkConnectAllowed
                    } else {
                        MonitorEventKind::NetworkConnectDenied
                    },
                    actor: "network-monitor".to_string(),
                    target,
                    evidence: serde_json::json!({
                        "direction": obs.direction,
                        "protocol": obs.protocol,
                        "remote_host": obs.remote_host,
                        "remote_port": obs.remote_port,
                        "url": obs.url,
                        "allowed": obs.allowed,
                        "bytes_sent": obs.bytes_sent,
                    }),
                }
            })
            .collect()
    }

    pub fn record_socket_attempt(&self, addr: SocketAddr, use_case: SocketAddrUse, allowed: bool) {
        self.record(NetworkObservation {
            direction: NetworkDirection::Outbound,
            protocol: socket_use_protocol(use_case).to_string(),
            remote_host: addr.ip().to_string(),
            remote_port: Some(addr.port()),
            url: None,
            allowed,
            bytes_sent: None,
        });
    }

    pub fn record_http_proxy_request(
        &self,
        method: &str,
        target: &str,
        allowed: bool,
        bytes_sent: Option<usize>,
    ) {
        let (host, port, url) = parse_http_target(target);
        self.record(NetworkObservation {
            direction: NetworkDirection::Outbound,
            protocol: if method.eq_ignore_ascii_case("CONNECT") {
                "tcp-connect".to_string()
            } else {
                "http".to_string()
            },
            remote_host: host,
            remote_port: port,
            url,
            allowed,
            bytes_sent,
        });
    }

    /// Start a localhost egress proxy that logs HTTP/CONNECT attempts and denies by default.
    pub fn start_egress_proxy(self: &Arc<Self>) -> std::io::Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(false)?;
        let port = listener.local_addr()?.port();
        let collector = Arc::clone(self);

        let handle = thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let collector = Arc::clone(&collector);
                thread::spawn(move || {
                    let _ = handle_proxy_connection(stream, collector);
                });
            }
        });

        *self.proxy_handle.lock().unwrap() = Some(handle);
        Ok(port)
    }
}

pub fn observations_from_http_intents(stdout: &str, stderr: &str) -> Vec<NetworkObservation> {
    let mut out = Vec::new();
    for line in stdout.lines().chain(stderr.lines()) {
        let trimmed = line.trim();
        for prefix in ["HTTP_FETCH:", "FETCH:", "HTTP:"] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                let url = rest.trim();
                if url.is_empty() {
                    continue;
                }
                let (host, port, parsed_url) = parse_http_target(url);
                out.push(NetworkObservation {
                    direction: NetworkDirection::Outbound,
                    protocol: "http-intent".to_string(),
                    remote_host: host,
                    remote_port: port,
                    url: parsed_url,
                    allowed: false,
                    bytes_sent: None,
                });
            }
        }
    }
    out
}

fn socket_use_protocol(use_case: SocketAddrUse) -> &'static str {
    match use_case {
        SocketAddrUse::TcpBind => "tcp-bind",
        SocketAddrUse::TcpConnect => "tcp-connect",
        SocketAddrUse::UdpBind => "udp-bind",
        SocketAddrUse::UdpConnect => "udp-connect",
        SocketAddrUse::UdpOutgoingDatagram => "udp-send",
    }
}

fn parse_http_target(target: &str) -> (String, Option<u16>, Option<String>) {
    if let Some(rest) = target.strip_prefix("http://") {
        return split_host_port_url(rest, Some(format!("http://{rest}")));
    }
    if let Some(rest) = target.strip_prefix("https://") {
        return split_host_port_url(rest, Some(format!("https://{rest}")));
    }
    split_host_port_url(target, None)
}

fn split_host_port_url(target: &str, url: Option<String>) -> (String, Option<u16>, Option<String>) {
    let host_port = target.split('/').next().unwrap_or(target);
    if let Some((host, port)) = host_port.rsplit_once(':') {
        if let Ok(port) = port.parse::<u16>() {
            return (host.to_string(), Some(port), url);
        }
    }
    (host_port.to_string(), None, url)
}

fn handle_proxy_connection(
    mut stream: TcpStream,
    collector: Arc<NetworkCollector>,
) -> std::io::Result<()> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buf[..n]);
    let mut lines = request.lines();
    let Some(request_line) = lines.next() else {
        return Ok(());
    };

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("UNKNOWN");
    let target = parts.next().unwrap_or("");

    collector.record_http_proxy_request(method, target, false, Some(n));

    let body = "connection denied by sandbox network monitor\r\n";
    let response = format!(
        "HTTP/1.1 403 Forbidden\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_http_intent_observations() {
        let stdout = "HTTP_FETCH: https://evil.example/c2/beacon\n";
        let obs = observations_from_http_intents(stdout, "");
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].protocol, "http-intent");
        assert_eq!(obs[0].remote_host, "evil.example");
    }

    #[test]
    fn egress_proxy_records_denied_connect() {
        let collector = Arc::new(NetworkCollector::new());
        let port = collector.start_egress_proxy().expect("bind proxy");

        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect to proxy");
        stream
            .write_all(b"CONNECT c2.evil.example:443 HTTP/1.1\r\nHost: c2.evil.example:443\r\n\r\n")
            .expect("write CONNECT");

        thread::sleep(std::time::Duration::from_millis(100));

        let obs = collector.observations();
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].remote_host, "c2.evil.example");
        assert!(!obs[0].allowed);
    }

    #[test]
    fn collector_emits_network_connect_sources() {
        let collector = NetworkCollector::new();
        collector.record(NetworkObservation {
            direction: NetworkDirection::Outbound,
            protocol: "tcp-connect".to_string(),
            remote_host: "203.0.113.10".to_string(),
            remote_port: Some(443),
            url: None,
            allowed: false,
            bytes_sent: None,
        });

        let sources = collector.as_taint_sources();
        assert_eq!(sources.len(), 1);
        assert!(matches!(sources[0], TaintSource::NetworkConnect { .. }));
    }
}
