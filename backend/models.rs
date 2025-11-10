use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Post models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub body: String,
    pub published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author_id: Uuid,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostSummary {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub published_at: DateTime<Utc>,
    pub reading_time: String,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPostSummary {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub published_at: String,
    pub reading_time: String,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePostRequest {
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub body: String,
    pub tags: Vec<Uuid>, // Tag IDs
    pub published: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePostRequest {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub excerpt: Option<String>,
    pub body: Option<String>,
    pub tags: Option<Vec<Uuid>>,
}

// Tag models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub color: String, // Catppuccin color name
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: String,
}

// Post-Tag relationship
#[derive(Debug, Clone)]
pub struct PostTag {
    pub post_id: Uuid,
    pub tag_id: Uuid,
}

// Auth models
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user id
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

// Markdown preview
#[derive(Debug, Serialize, Deserialize)]
pub struct MarkdownPreviewRequest {
    pub markdown: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarkdownPreviewResponse {
    pub html: String,
    pub reading_time: String,
}

// Draft model (for unpublished posts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub body: String,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Session model for authentication
#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// Asset model for images and files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    #[serde(skip)]
    pub data: Vec<u8>,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

// Wiki-link tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostLink {
    pub id: Uuid,
    pub from_post_id: Uuid,
    pub to_post_slug: String,
    pub link_text: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Statistics model
#[derive(Debug, Serialize, Deserialize)]
pub struct PostStats {
    pub published_count: i64,
    pub draft_count: i64,
    pub total_count: i64,
    pub author_count: i64,
    pub tag_count: i64,
}

// Search result model
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub posts: Vec<PostSummary>,
    pub total: usize,
    pub query: String,
}

// Pagination model
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
        }
    }
}

// API Response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}
