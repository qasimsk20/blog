use sqlx::PgPool;
use std::sync::Arc;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool
    pub pool: PgPool,
    /// JWT secret for token signing and verification
    pub jwt_secret: String,
    /// Optional: Frontend URL for CORS configuration
    pub frontend_url: Option<String>,
}

impl AppState {
    /// Create a new application state
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        Self {
            pool,
            jwt_secret,
            frontend_url: None,
        }
    }

    /// Create a new application state with frontend URL
    pub fn with_frontend_url(pool: PgPool, jwt_secret: String, frontend_url: String) -> Self {
        Self {
            pool,
            jwt_secret,
            frontend_url: Some(frontend_url),
        }
    }
}

/// Enable extraction of AppState from axum's state
///
/// Note:
/// We intentionally avoid implementing `FromRef<AppState> for AppState`
/// because axum-core already provides a blanket `FromRef<T> for T` impl.
/// This prevents conflicting trait implementations while still allowing
/// `AppState` (or `Arc<AppState>`) to be extracted as shared state.

// If the router stores Arc<AppState>, allow extracting an owned AppState as well.
impl shuttle_axum::axum::extract::FromRef<std::sync::Arc<AppState>> for AppState {
    fn from_ref(state: &std::sync::Arc<AppState>) -> AppState {
        (*state).as_ref().clone()
    }
}
