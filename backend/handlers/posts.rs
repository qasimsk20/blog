use shuttle_axum::axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db,
    error::AppError,
    markdown::{extract_links, render_obsidian_markdown, strip_first_heading},
    models::{Post, PostSummary},
    state::AppState,
};

/// List all published posts
pub async fn list_posts(State(state): State<Arc<AppState>>) -> Result<Json<Vec<PostSummary>>, AppError> {
    let posts = db::list_published_posts(&state.pool).await?;
    Ok(Json(posts))
}

/// Get a single published post by slug
pub async fn get_post(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<PostResponse>, AppError> {
    let post = db::get_post_by_slug(&state.pool, &slug)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Post '{}' not found", slug)))?;

    // Strip the first heading from body for rendering
    let body = strip_first_heading(&post.body);
    tracing::info!("Original body starts with: {:?}", &post.body.chars().take(50).collect::<String>());
    tracing::info!("Stripped body starts with: {:?}", &body.chars().take(50).collect::<String>());

    // Render the markdown content to HTML
    let html = render_obsidian_markdown(&body);

    // Extract wiki-links for potential backlinks
    let links = extract_links(&post.body);

    // Get related posts by tags
    let related = db::get_related_posts(&state.pool, post.id, 5).await?;

    // Get adjacent posts (previous and next by date)
    let adjacent = get_adjacent_posts(&state.pool, &post).await?;

    let response = PostResponse {
        id: post.id,
        slug: post.slug,
        title: post.title,
        excerpt: post.excerpt,
        body,
        html,
        published_at: post.published_at,
        created_at: post.created_at,
        updated_at: post.updated_at,
        tags: post.tags,
        links,
        related,
        adjacent,
    };

    Ok(Json(response))
}

/// Response structure for a single post with additional data
#[derive(serde::Serialize)]
pub struct PostResponse {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub body: String,
    pub html: String,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<crate::models::Tag>,
    pub links: Vec<String>,
    pub related: Vec<PostSummary>,
    pub adjacent: AdjacentPosts,
}

#[derive(serde::Serialize)]
pub struct AdjacentPosts {
    pub previous: Option<PostSummary>,
    pub next: Option<PostSummary>,
}

/// Get adjacent posts (previous and next by publication date)
async fn get_adjacent_posts(
    pool: &sqlx::PgPool,
    current_post: &Post,
) -> Result<AdjacentPosts, AppError> {
    // For now, return None for both - can be implemented later
    Ok(AdjacentPosts { previous: None, next: None })
}

/// Search published posts
pub async fn search_posts(
    State(state): State<AppState>,
    query: String,
) -> Result<Json<Vec<PostSummary>>, AppError> {
    if query.trim().is_empty() {
        return Ok(Json(vec![]));
    }

    let posts = db::search_posts(&state.pool, &query)
        .await?
        .into_iter()
        .filter(|p| p.published)
        .map(|p| PostSummary {
            id: p.id,
            slug: p.slug,
            title: p.title,
            excerpt: p.excerpt,
            published_at: p.published_at.unwrap_or(p.created_at),
            reading_time: crate::markdown::calculate_reading_time(&p.body),
            tags: p.tags,
        })
        .collect();

    Ok(Json(posts))
}

/// Get posts by tag
pub async fn get_posts_by_tag(
    State(state): State<AppState>,
    Path(tag_name): Path<String>,
) -> Result<Json<Vec<PostSummary>>, AppError> {
    let posts = db::get_posts_by_tag(&state.pool, &tag_name).await?;
    Ok(Json(posts))
}


