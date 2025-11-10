use shuttle_axum::axum::extract::{Path, State};
use shuttle_axum::axum::Json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{auth::AuthUser, db, error::AppError, models::{CreateTagRequest, Tag}, state::AppState};

/// List all tags
pub async fn list_tags(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Tag>>, AppError> {
    let tags = db::list_tags(&state.pool).await?;
    Ok(Json(tags))
}

/// Get tag statistics (post count per tag)
pub async fn get_tag_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TagStats>>, AppError> {
    let stats = db::get_tag_stats(&state.pool).await?;
    Ok(Json(stats))
}

#[derive(serde::Serialize)]
pub struct TagStats {
    pub tag: Tag,
    pub post_count: usize,
}

/// Create a new tag (admin only)
pub async fn create_tag(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<Tag>, AppError> {
    let tag = db::create_tag(&state.pool, req).await?;
    Ok(Json(tag))
}

/// Delete a tag (admin only)
pub async fn delete_tag(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(tag_id): Path<String>,
) -> Result<Json<()>, AppError> {
    let id = Uuid::parse_str(&tag_id)
        .map_err(|_| AppError::BadRequest("Invalid tag ID".to_string()))?;
    db::delete_tag(&state.pool, id).await?;
    Ok(Json(()))
}
