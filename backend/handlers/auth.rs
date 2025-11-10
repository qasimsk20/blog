use shuttle_axum::axum::{
    extract::{FromRef, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use serde_json::json;
use tokio::time::{sleep, Duration};

use crate::{
    auth::{generate_jwt, verify_password},
    db,
    error::AppError,
    models::{LoginRequest, LoginResponse, UserInfo},
    state::AppState,
};

/// Real login endpoint (backed by Postgres users table).
/// - Verifies username/password with Argon2
/// - Issues a JWT on success
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let username = payload.username.trim();
    let password = payload.password;

    if username.is_empty() || password.is_empty() {
        return Err(AppError::BadRequest(
            "username and password are required".into(),
        ));
    }

    // Fetch user
    let user = db::get_user_by_username(&state.pool, username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;

    // Verify password
    let verified = verify_password(&password, &user.password_hash)
        .map_err(|_| AppError::Unauthorized("Invalid credentials".into()))?;

    if !verified {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    // Issue JWT
    let token = generate_jwt(user.id, &user.username, &state.jwt_secret)?;

    let res = LoginResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
        },
    };

    tracing::info!("user '{}' logged in", res.user.username);

    Ok(Json(res))
}

/// Decoy login endpoint (always fails).
/// - Lives under the "admin" path to attract scanners
/// - Always returns 401 with a small delay
pub async fn decoy_login(Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    // Add a small randomized delay to make enumeration harder
    let delay_ms = 300 + (payload.username.len() as u64 % 400);
    sleep(Duration::from_millis(delay_ms)).await;

    tracing::warn!("decoy login attempt for user '{}'", payload.username);

    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "Invalid credentials",
            "message": "The username or password you entered is incorrect."
        })),
    )
}
