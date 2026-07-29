use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use futures_util::future::BoxFuture;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct RateLimitLayer {
    rps: u64,
    burst: u32,
    store: Arc<Mutex<HashMap<String, TokenBucket>>>,
}

struct TokenBucket {
    tokens: f64,
    last: Instant,
}

impl RateLimitLayer {
    pub fn new(rps: u64, burst: u32) -> Self {
        Self {
            rps: rps.max(1),
            burst: burst.max(1),
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            rps: self.rps,
            burst: self.burst,
            store: self.store.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    rps: u64,
    burst: u32,
    store: Arc<Mutex<HashMap<String, TokenBucket>>>,
}

impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let key = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
            .unwrap_or_else(|| "local".into());

        let allowed = {
            let mut map = self.store.lock().unwrap();
            let now = Instant::now();
            let bucket = map.entry(key).or_insert(TokenBucket {
                tokens: self.burst as f64,
                last: now,
            });
            let elapsed = now.duration_since(bucket.last).as_secs_f64();
            bucket.tokens = (bucket.tokens + elapsed * self.rps as f64).min(self.burst as f64);
            bucket.last = now;
            if bucket.tokens >= 1.0 {
                bucket.tokens -= 1.0;
                true
            } else {
                false
            }
        };

        let mut inner = self.inner.clone();
        Box::pin(async move {
            if !allowed {
                let body = Body::from(r#"{"error":"rate limited","code":"rate_limited"}"#);
                let mut res = Response::new(body);
                *res.status_mut() = StatusCode::TOO_MANY_REQUESTS;
                res.headers_mut().insert(
                    axum::http::header::CONTENT_TYPE,
                    "application/json".parse().unwrap(),
                );
                res.headers_mut().insert(
                    axum::http::header::RETRY_AFTER,
                    "1".parse().unwrap(),
                );
                return Ok(res);
            }
            // opportunistic cleanup
            {
                // no-op placeholder — HashMap grows slowly; production would use Redis
                let _ = Duration::from_secs(1);
            }
            inner.call(req).await
        })
    }
}
