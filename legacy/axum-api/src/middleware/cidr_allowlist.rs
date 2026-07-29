//! Optional source-IP allowlist (defense in depth behind VPN/firewall).

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use axum::response::IntoResponse;
use futures_util::future::BoxFuture;
use tower::{Layer, Service};

#[derive(Clone, Debug)]
pub struct CidrAllowlistLayer {
    networks: Arc<Vec<Ipv4Network>>,
}

impl CidrAllowlistLayer {
    /// Empty list disables filtering. Parse entries like `10.8.0.0/24` or `127.0.0.1/32`.
    pub fn from_cidrs(cidrs: &[String]) -> Result<Self, String> {
        let mut networks = Vec::new();
        for raw in cidrs {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            networks.push(Ipv4Network::parse(trimmed)?);
        }
        Ok(Self {
            networks: Arc::new(networks),
        })
    }

    pub fn is_enabled(&self) -> bool {
        !self.networks.is_empty()
    }

    pub fn allows(&self, ip: IpAddr) -> bool {
        if self.networks.is_empty() {
            return true;
        }
        match ip {
            IpAddr::V4(v4) => self.networks.iter().any(|n| n.contains(v4)),
            // IPv6 not configured — deny when allowlist is active
            IpAddr::V6(_) => false,
        }
    }
}

impl<S> Layer<S> for CidrAllowlistLayer {
    type Service = CidrAllowlistService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CidrAllowlistService {
            inner,
            layer: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CidrAllowlistService<S> {
    inner: S,
    layer: CidrAllowlistLayer,
}

impl<S> Service<Request<Body>> for CidrAllowlistService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let layer = self.layer.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            if layer.is_enabled() {
                let ip = client_ip(&req);
                if let Some(ip) = ip {
                    if !layer.allows(ip) {
                        tracing::warn!(%ip, "blocked by ALLOWED_CIDRS");
                        return Ok((StatusCode::FORBIDDEN, "access denied: not on allowlisted network").into_response());
                    }
                } else {
                    tracing::warn!("blocked by ALLOWED_CIDRS: could not determine client IP");
                    return Ok((StatusCode::FORBIDDEN, "access denied: unknown client IP").into_response());
                }
            }
            inner.call(req).await
        })
    }
}

fn client_ip(req: &Request<Body>) -> Option<IpAddr> {
    // Prefer direct peer when available via ConnectInfo (set by axum::serve with into_make_service_with_connect_info)
    if let Some(addr) = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
    {
        return Some(addr.0.ip());
    }
    // Fallback: first X-Forwarded-For hop (only trust behind a private reverse proxy)
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.trim().parse().ok())
        })
}

#[derive(Clone, Debug)]
struct Ipv4Network {
    network: u32,
    mask: u32,
}

impl Ipv4Network {
    fn parse(cidr: &str) -> Result<Self, String> {
        let (addr_s, prefix_s) = cidr
            .split_once('/')
            .ok_or_else(|| format!("invalid CIDR '{cidr}' (expected a.b.c.d/nn)"))?;
        let addr: Ipv4Addr = addr_s
            .parse()
            .map_err(|_| format!("invalid IPv4 in CIDR '{cidr}'"))?;
        let prefix: u32 = prefix_s
            .parse()
            .map_err(|_| format!("invalid prefix in CIDR '{cidr}'"))?;
        if prefix > 32 {
            return Err(format!("prefix out of range in CIDR '{cidr}'"));
        }
        let mask = if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix)
        };
        let network = u32::from(addr) & mask;
        Ok(Self { network, mask })
    }

    fn contains(&self, ip: Ipv4Addr) -> bool {
        (u32::from(ip) & self.mask) == self.network
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cidr_contains() {
        let n = Ipv4Network::parse("10.8.0.0/24").unwrap();
        assert!(n.contains(Ipv4Addr::new(10, 8, 0, 1)));
        assert!(!n.contains(Ipv4Addr::new(10, 8, 1, 1)));
        let layer = CidrAllowlistLayer::from_cidrs(&[
            "10.8.0.0/24".into(),
            "192.168.100.0/24".into(),
            "127.0.0.1/32".into(),
        ])
        .unwrap();
        assert!(layer.allows(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
        assert!(layer.allows(IpAddr::V4(Ipv4Addr::new(192, 168, 100, 50))));
        assert!(!layer.allows(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    }
}
