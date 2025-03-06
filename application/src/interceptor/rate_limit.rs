use anyhow::Result;

use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};

// Shared state for rate limiting
#[derive(Debug, Clone)]
pub struct AppState {
    pub limit: u32,
    // Simple in-memory rate limiter (replace with a more robust solution)
    pub ip_counts: Arc<Mutex<std::collections::HashMap<IpAddr, u32>>>,
}

impl AppState {
    pub fn new(limit: u32) -> Self {
        Self {
            limit,
            ip_counts: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
}

// Middleware for rate limiting by IP
pub async fn rate_limit(
    state: State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let ip = req.extensions().get::<IpAddr>().cloned().ok_or((
        StatusCode::BAD_REQUEST,
        "Could not get IP address".to_string(),
    ))?;

    let mut ip_counts = state.ip_counts.lock().unwrap();
    let count = ip_counts.entry(ip).or_insert(0);
    *count += 1;

    if *count > state.limit {
        Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded".to_string(),
        ))
    } else {
        Ok(next.run(req).await)
    }
}
