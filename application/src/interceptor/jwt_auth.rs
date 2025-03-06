use anyhow::Result;

use axum::{
    body::Body, extract::Request, http::StatusCode, middleware::Next, response::IntoResponse,
};

// Middleware for JWT authentication
pub async fn jwt_auth(
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let auth_header = req.headers().get("authorization");

    match auth_header {
        Some(header) => {
            let token = header.to_str().unwrap().replace("Bearer ", ""); // Remove "Bearer " prefix

            // TODO: Implement JWT verification logic here
            if verify_jwt(&token) {
                Ok(next.run(req).await)
            } else {
                Err((StatusCode::UNAUTHORIZED, "Invalid JWT".to_string()))
            }
        }
        // None => Err((StatusCode::UNAUTHORIZED, "Missing JWT".to_string())),
        // FIXME: Uncomment the line above and comment the line below to require a JWT
        None => Ok(next.run(req).await),
    }
}

// Dummy JWT verification function (replace with a real implementation)
fn verify_jwt(_token: &str) -> bool {
    // Replace with your actual JWT verification logic
    // This is just a placeholder
    true // Always returns true for demonstration purposes
}
