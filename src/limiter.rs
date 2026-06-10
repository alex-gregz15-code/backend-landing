use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(Clone)]
pub struct ConcurrencyLimiter {
    semaphore: Arc<Semaphore>,
}

impl ConcurrencyLimiter {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    pub async fn acquire(&self) -> Option<OwnedSemaphorePermit> {
        self.semaphore.clone().acquire_owned().await.ok()
    }
}

pub async fn enforce_concurrency(
    limiter: ConcurrencyLimiter,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    if limiter.acquire().await.is_some() {
        Ok(next.run(req).await)
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, "Too many requests".to_string()))
    }
}
