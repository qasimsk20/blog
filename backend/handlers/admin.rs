use shuttle_axum::axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use serde_json::json;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::AppError,
    markdown::{calculate_reading_time, extract_tags, render_obsidian_markdown},
    models::{
        AdminPostSummary, CreatePostRequest, CreateTagRequest, MarkdownPreviewRequest, MarkdownPreviewResponse, Post,
        Tag, UpdatePostRequest,
    },
    state::AppState,
};

/// Create a new blog post
pub async fn create_post(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<Post>), AppError> {
    // Validate slug format
    if !is_valid_slug(&req.slug) {
        return Err(AppError::BadRequest(
            "Invalid slug format. Use lowercase letters, numbers, and hyphens only.".to_string(),
        ));
    }

    // Check if slug already exists
    if let Some(_) = db::get_post_by_slug(&state.pool, &req.slug).await? {
        return Err(AppError::BadRequest(format!(
            "A post with slug '{}' already exists",
            req.slug
        )));
    }

    // Extract tags from markdown content if not explicitly provided
    let auto_tags = extract_tags(&req.body);

    // Create the post
    let post = db::create_post(&state.pool, req, user.user_id).await?;

    // Log the creation
    tracing::info!("Post created: {} by user {}", post.slug, user.username);

    Ok((StatusCode::CREATED, Json(post)))
}

/// Update an existing blog post
pub async fn update_post(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(slug): Path<String>,
    Json(req): Json<UpdatePostRequest>,
) -> Result<Json<Post>, AppError> {
    // Check if post exists and user owns it (including unpublished posts)
    let existing = db::get_post_by_slug_any(&state.pool, &slug)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    if existing.author_id != user.user_id {
        return Err(AppError::Forbidden(
            "You don't have permission to update this post".to_string(),
        ));
    }

    // Validate new slug if provided
    if let Some(ref new_slug) = req.slug {
        if !is_valid_slug(new_slug) {
            return Err(AppError::BadRequest(
                "Invalid slug format. Use lowercase letters, numbers, and hyphens only."
                    .to_string(),
            ));
        }

        // Check if new slug is already taken by another post
        if new_slug != &existing.slug {
            if let Some(_) = db::get_post_by_slug(&state.pool, new_slug).await? {
                return Err(AppError::BadRequest(format!(
                    "A post with slug '{}' already exists",
                    new_slug
                )));
            }
        }
    }

    // Update the post
    let updated_post = db::update_post(&state.pool, existing.id, req).await?;

    tracing::info!(
        "Post updated: {} by user {}",
        updated_post.slug,
        user.username
    );

    Ok(Json(updated_post))
}

/// Delete a blog post
pub async fn delete_post(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<StatusCode, AppError> {
    tracing::info!("Delete request for slug: {} by user {}", slug, user.username);
    
    // Check if post exists and user owns it (including unpublished posts)
    let existing = db::get_post_by_slug_any(&state.pool, &slug)
        .await?
        .ok_or_else(|| {
            tracing::warn!("Post not found: {}", slug);
            AppError::NotFound("Post not found".to_string())
        })?;

    tracing::info!("Found post: {} with author_id: {}, user_id: {}", 
        existing.slug, existing.author_id, user.user_id);

    if existing.author_id != user.user_id {
        return Err(AppError::Forbidden(
            "You don't have permission to delete this post".to_string(),
        ));
    }

    // Delete the post
    db::delete_post(&state.pool, existing.id).await?;

    tracing::info!("Post deleted: {} by user {}", existing.slug, user.username);

    Ok(StatusCode::NO_CONTENT)
}

/// Publish a draft post
pub async fn publish_post(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<Post>, AppError> {
    // Check if post exists and user owns it (including unpublished posts)
    let existing = db::get_post_by_slug_any(&state.pool, &slug)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    if existing.author_id != user.user_id {
        return Err(AppError::Forbidden(
            "You don't have permission to publish this post".to_string(),
        ));
    }

    if existing.published {
        return Err(AppError::BadRequest(
            "Post is already published".to_string(),
        ));
    }

    // Publish the post
    let published_post = db::publish_post(&state.pool, existing.id).await?;

    tracing::info!(
        "Post published: {} by user {}",
        published_post.slug,
        user.username
    );

    Ok(Json(published_post))
}

/// Unpublish a published post
pub async fn unpublish_post(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<Post>, AppError> {
    // Check if post exists and user owns it
    let existing = db::get_post_by_slug(&state.pool, &slug)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    if existing.author_id != user.user_id {
        return Err(AppError::Forbidden(
            "You don't have permission to unpublish this post".to_string(),
        ));
    }

    if !existing.published {
        return Err(AppError::BadRequest(
            "Post is already unpublished".to_string(),
        ));
    }

    // Unpublish the post
    let unpublished_post = db::unpublish_post(&state.pool, existing.id).await?;

    tracing::info!(
        "Post unpublished: {} by user {}",
        unpublished_post.slug,
        user.username
    );

    Ok(Json(unpublished_post))
}

/// Preview markdown content
pub async fn preview_markdown(
    _user: AuthUser,
    Json(req): Json<MarkdownPreviewRequest>,
) -> Result<Json<MarkdownPreviewResponse>, AppError> {
    // Render the markdown with Obsidian features
    let html = render_obsidian_markdown(&req.markdown);

    // Calculate reading time
    let reading_time = calculate_reading_time(&req.markdown);

    Ok(Json(MarkdownPreviewResponse { html, reading_time }))
}

/// Create a new tag
pub async fn create_tag(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<Tag>), AppError> {
    // Validate tag name
    if !is_valid_tag_name(&req.name) {
        return Err(AppError::BadRequest(
            "Invalid tag name. Use lowercase letters, numbers, and hyphens only.".to_string(),
        ));
    }

    // Validate color is a valid Catppuccin color
    if !is_valid_catppuccin_color(&req.color) {
        return Err(AppError::BadRequest(
            "Invalid color. Must be a valid Catppuccin color name.".to_string(),
        ));
    }

    // Check if tag already exists
    let existing_tags = db::list_tags(&state.pool).await?;
    if existing_tags.iter().any(|t| t.name == req.name) {
        return Err(AppError::BadRequest(format!(
            "Tag '{}' already exists",
            req.name
        )));
    }

    // Create the tag
    let tag = db::create_tag(&state.pool, req).await?;

    tracing::info!("Tag created: {} by user {}", tag.name, user.username);

    Ok((StatusCode::CREATED, Json(tag)))
}

/// Update an existing tag
pub async fn update_tag(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<Tag>, AppError> {
    // Validate tag name
    if !is_valid_tag_name(&req.name) {
        return Err(AppError::BadRequest(
            "Invalid tag name. Use lowercase letters, numbers, and hyphens only.".to_string(),
        ));
    }

    // Validate color is a valid Catppuccin color
    if !is_valid_catppuccin_color(&req.color) {
        return Err(AppError::BadRequest(
            "Invalid color. Must be a valid Catppuccin color name.".to_string(),
        ));
    }

    // Update the tag
    let tag = db::update_tag(&state.pool, id, req).await?;

    tracing::info!("Tag updated: {} by user {}", tag.name, user.username);

    Ok(Json(tag))
}

/// Delete a tag
pub async fn delete_tag(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Delete the tag (will cascade to remove from post_tags)
    db::delete_tag(&state.pool, id).await?;

    tracing::info!("Tag deleted: {} by user {}", id, user.username);

    Ok(StatusCode::NO_CONTENT)
}

/// Get all posts (including unpublished) for admin
pub async fn list_all_posts(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
) -> Result<Json<Vec<AdminPostSummary>>, AppError> {
    let posts = db::list_all_posts(&state.pool).await?;
    let summaries: Vec<AdminPostSummary> = posts
        .into_iter()
        .map(|p| AdminPostSummary {
            id: p.id.to_string(),
            slug: p.slug,
            title: p.title,
            excerpt: p.excerpt,
            published_at: p.published_at.unwrap_or(p.created_at).to_rfc3339(),
            reading_time: calculate_reading_time(&p.body),
            tags: p.tags,
        })
        .collect();
    Ok(Json(summaries))
}

/// Get post statistics for admin dashboard
pub async fn get_post_stats(State(state): State<Arc<AppState>>, _user: AuthUser) -> impl IntoResponse {
    let stats = match db::get_post_stats(&state.pool).await {
        Ok(stats) => stats,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to fetch statistics"
                })),
            )
        }
    };

    (StatusCode::OK, Json(stats))
}

/// Search posts with full-text search
pub async fn search_posts(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    query: String,
) -> Result<Json<Vec<Post>>, AppError> {
    let posts = db::search_posts(&state.pool, &query).await?;
    Ok(Json(posts))
}

// Helper functions

fn is_valid_slug(slug: &str) -> bool {
    slug.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--")
}

fn is_valid_tag_name(name: &str) -> bool {
    name.len() <= 100
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn is_valid_catppuccin_color(color: &str) -> bool {
    matches!(
        color,
        "rosewater"
            | "flamingo"
            | "pink"
            | "mauve"
            | "red"
            | "maroon"
            | "peach"
            | "yellow"
            | "green"
            | "teal"
            | "sky"
            | "sapphire"
            | "blue"
            | "lavender"
            | "surface0"
            | "surface1"
            | "surface2"
            | "overlay0"
            | "overlay1"
            | "overlay2"
            | "subtext0"
            | "subtext1"
            | "text"
            | "base"
            | "mantle"
            | "crust"
    )
}
