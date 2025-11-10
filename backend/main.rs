use shuttle_axum::axum::{
    extract::{Query, State},
    http::{HeaderValue, Method},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use shuttle_axum::{ShuttleAxum, AxumService};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

// Bring our modules into scope
mod auth;
mod db;
mod error;
mod handlers;
mod markdown;
mod models;
mod state;

// Shared application state handle
type SharedState = Arc<state::AppState>;

// Basic health handler
async fn health() -> &'static str {
    "ok"
}

// Optional: simple probe endpoint to sanity check DB connectivity
async fn db_probe(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, error::AppError> {
    let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&state.pool).await?;
    Ok(Json(serde_json::json!({ "db": row.0 })))
}

// Public search handler (?q=) mapping DB rows to summaries
#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

async fn public_search(
    State(state): State<SharedState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<models::PostSummary>>, error::AppError> {
    let q = params.q.unwrap_or_default();
    if q.trim().is_empty() {
        return Ok(Json(vec![]));
    }

    let posts = db::search_posts(&state.pool, &q).await?;
    let summaries: Vec<models::PostSummary> = posts
        .into_iter()
        .filter(|p| p.published)
        .map(|p| models::PostSummary {
            id: p.id,
            slug: p.slug,
            title: p.title,
            excerpt: p.excerpt,
            published_at: p.published_at.unwrap_or(p.created_at),
            reading_time: markdown::calculate_reading_time(&p.body),
            tags: p.tags,
        })
        .collect();

    Ok(Json(summaries))
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> ShuttleAxum {

    // Get configuration from Shuttle secrets
    let database_url = secrets
        .get("DATABASE_URL")
        .expect("DATABASE_URL secret not set");
    let jwt_secret = secrets
        .get("JWT_SECRET")
        .unwrap_or_else(|| "development-secret-change-me".to_string());
    let admin_password = secrets
        .get("ADMIN_PASSWORD")
        .unwrap_or_else(|| "admin123".to_string());
    let cors_origins = secrets.get("CORS_ORIGINS");

    // Database connection
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    println!("Running database migrations...");
    sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .map_err(|e| anyhow::anyhow!("Migration error: {}", e))?
        .run(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Migration run error: {}", e))?;
    println!("✅ Migrations completed");

    // Initialize admin user
    println!("Creating admin user...");
    db::init_admin_user(&pool, &admin_password)
        .await
        .expect("Failed to initialize admin user");
    println!("✅ Admin user created");

    let app_state = Arc::new(state::AppState::new(pool, jwt_secret));

    // CORS
    let mut cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    // Prefer explicit allowlist via CORS_ORIGINS; else in dev allow any
    if let Some(list) = cors_origins {
        let allowed: Vec<HeaderValue> = list
            .split(',')
            .filter_map(|o| o.trim().parse().ok())
            .collect();
        if !allowed.is_empty() {
            cors = cors.allow_origin(allowed);
        } else {
            cors = cors.allow_origin(Any);
        }
    } else {
        // Development convenience; prefer to configure origins explicitly in production
        cors = cors.allow_origin(Any);
    }

    // Routers
    let public_api = Router::new()
        .route("/health", get(health))
        .route("/db-probe", get(db_probe))
        // Posts
        .route("/posts", get(handlers::posts::list_posts))
        .route("/posts/{slug}", get(handlers::posts::get_post))
        // Tags
        .route("/tags", get(handlers::tags::list_tags).post(handlers::tags::create_tag))
        .route("/tags/stats", get(handlers::tags::get_tag_stats))
        .route("/tags/{tag_id}", delete(handlers::tags::delete_tag))
        // Search
        .route("/search", get(public_search))
        // Auth
        .route("/auth/login", post(handlers::auth::login))
        // Decoy
        .route("/admin/login", post(handlers::auth::decoy_login))
        .with_state(app_state.clone());

    let admin_api = Router::new()
        // Posts (admin)
        .route(
            "/posts",
            post(handlers::admin::create_post).get(handlers::admin::list_all_posts),
        )
        .route(
            "/posts/{slug}",
            put(handlers::admin::update_post).delete(handlers::admin::delete_post),
        )
        .route("/posts/{slug}/publish", post(handlers::admin::publish_post))
        .route(
            "/posts/{slug}/unpublish",
            post(handlers::admin::unpublish_post),
        )
        .route("/stats", get(handlers::admin::get_post_stats))
        // Markdown preview
        .route("/preview", post(handlers::admin::preview_markdown))
        // Tags (admin)
        .route("/tags", post(handlers::admin::create_tag))
        .route(
            "/tags/{id}",
            put(handlers::admin::update_tag).delete(handlers::admin::delete_tag),
        )
        .with_state(app_state.clone());

    let app = Router::new()
        .nest("/api", public_api)
        .nest("/api/sayyidati", admin_api)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    Ok(AxumService::from(app))
}
