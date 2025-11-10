use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use sqlx::{postgres::PgRow, PgPool, Row};
use uuid::Uuid;

use crate::{
    auth,
    models::{
        CreatePostRequest, CreateTagRequest, Post, PostSummary, Tag, UpdatePostRequest, User,
    },
};

/// Initialize admin user if it doesn't exist
pub async fn init_admin_user(pool: &PgPool, password: &str) -> Result<()> {
    let existing = sqlx::query(
        "SELECT id, username, password_hash, created_at, updated_at FROM users WHERE username = $1",
    )
    .bind("admin")
    .fetch_optional(pool)
    .await?;

    let password_hash = auth::hash_password(password)?;

    if existing.is_none() {
        let id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO users (id, username, password_hash, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind("admin")
        .bind(password_hash)
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(pool)
        .await?;

        tracing::info!("Admin user created successfully");
    } else {
        // Update existing admin password
        sqlx::query(
            "UPDATE users SET password_hash = $1, updated_at = $2 WHERE username = $3",
        )
        .bind(password_hash)
        .bind(Utc::now())
        .bind("admin")
        .execute(pool)
        .await?;

        tracing::info!("Admin password updated successfully");
    }

    Ok(())
}

/// Seed sample posts for demonstration
pub async fn seed_sample_posts(pool: &PgPool, author_id: Uuid) -> Result<()> {
    // Check if we already have posts
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    if count > 0 {
        return Ok(()); // Already have posts, skip seeding
    }

    // Create sample tags first
    let haskell_tag = create_tag(
        pool,
        CreateTagRequest {
            name: "haskell".to_string(),
            color: "lavender".to_string(),
        },
    )
    .await?;

    let fp_tag = create_tag(
        pool,
        CreateTagRequest {
            name: "functional-programming".to_string(),
            color: "blue".to_string(),
        },
    )
    .await?;

    let philosophy_tag = create_tag(
        pool,
        CreateTagRequest {
            name: "code-philosophy".to_string(),
            color: "mauve".to_string(),
        },
    )
    .await?;

    let languages_tag = create_tag(
        pool,
        CreateTagRequest {
            name: "programming-languages".to_string(),
            color: "peach".to_string(),
        },
    )
    .await?;

    // Read the sample Haskell post
    let haskell_content = r#"# The Quiet Elegance of Haskell: Where Code Becomes Poetry

There's a moment every Haskell programmer remembers—the point where the syntax fades away and what remains feels less like programming and more like stating simple, timeless truths.

## Purity as a Superpower

In most languages, you write *instructions*: "do this, then that." Haskell is different. You write *definitions*: "what something *is*." This isn't semantic pedantry—it's a radical shift in perspective.

```haskell
-- Instead of "how to calculate a factorial"
factorial 0 = 1
factorial n = n * factorial (n - 1)
```

This isn't just concise; it's a direct translation of the mathematical definition. No loops, no mutable counters, no step-by-step choreography—just the essence.

## Types That Think With You

Haskell's type system isn't a prison; it's a collaborative partner. It catches errors before you even run your code, but more beautifully, it often guides you to the correct implementation.

```haskell
-- This type signature almost writes the function itself
safeHead :: [a] -> Maybe a
safeHead [] = Nothing
safeHead (x:xs) = Just x
```

The type `Maybe a` elegantly captures the possibility of absence without null pointers or exceptions. It's honesty encoded in the type system—calling this function *requires* handling the empty case.

## Laziness: The Art of Infinite Patience

Because Haskell evaluates expressions only when needed, you can work with infinite structures as naturally as finite ones:

```haskell
-- The Fibonacci sequence, defined for all time
fibs = 0 : 1 : zipWith (+) fibs (tail fibs)

-- Take just what you need
firstTenFibs = take 10 fibs  -- [0,1,1,2,3,5,8,13,21,34]
```

This isn't a clever trick—it's the natural consequence of separating *definition* from *execution*. You describe what the Fibonacci sequence *is*, not how to compute it incrementally.

## Composition as Default

Haskell's greatest elegance lies in how functions compose. The `.` operator isn't just syntax—it's a philosophy:

```haskell
-- These are identical
result1 = f (g (h x))
result2 = (f . g . h) x
```

This tiny operator transforms nested parentheses into a pipeline of transformations, reading left to right as a story of data flowing through transformations.

## The Learning Curve Is Real, But...

Make no mistake: Haskell is hard. The leap from imperative thinking is significant. You'll wrestle with monads, functors, and the dreaded "pattern match failure." But here's the secret: the struggle *is* the elegance. Haskell forces you to confront complexity rather than bury it under syntactic sugar.

When you finally internalize that a monad is just a monoid in the category of endofunctors (joking! nobody actually thinks that), you realize it's simply a pattern for sequencing computations—a pattern so useful it appears everywhere, from lists to I/O to error handling.

## Final Thought

Elegance in Haskell isn't about brevity or clever one-liners. It's about how the language gets out of the way of your ideas. It asks: "What are you really trying to say?" and then gives you the tools to say it without distortion.

In a world of ever-increasing complexity, Haskell whispers a radical promise: **the answer isn't more features, but better constraints.** And in those constraints, we find a kind of freedom that's increasingly rare—a freedom to think clearly, to compose confidently, and to write code that might still make sense in twenty years.

That is elegance."#;

    // Create the Haskell elegance post
    let haskell_post = CreatePostRequest {
        slug: "haskell-elegance".to_string(),
        title: "The Quiet Elegance of Haskell: Where Code Becomes Poetry".to_string(),
        excerpt: "There's a moment every Haskell programmer remembers—the point where the syntax fades away and what remains feels less like programming and more like stating simple, timeless truths.".to_string(),
        body: haskell_content.to_string(),
        tags: vec![haskell_tag.id, fp_tag.id, philosophy_tag.id, languages_tag.id],
        published: true,
    };

    create_post(pool, haskell_post, author_id).await?;

    tracing::info!("Sample posts seeded successfully");
    Ok(())
}

/// Get user by username
pub async fn get_user_by_username(pool: &PgPool, username: &str) -> Result<Option<User>> {
    let row = sqlx::query(
        "SELECT id, username, password_hash, created_at, updated_at FROM users WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    let user = if let Some(row) = row {
        Some(User {
            id: row.get("id"),
            username: row.get("username"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    } else {
        None
    };

    Ok(user)
}

/// List all published posts with tags
pub async fn list_published_posts(pool: &PgPool) -> Result<Vec<PostSummary>> {
    let rows: Vec<PgRow> = sqlx::query(
        r#"
        SELECT
            p.id,
            p.slug,
            p.title,
            p.excerpt,
            p.body,
            p.published_at,
            COALESCE(
                (
                    SELECT json_agg(tag_obj ORDER BY (tag_obj->>'name'))
                    FROM (
                        SELECT json_build_object('id', t.id, 'name', t.name, 'color', t.color, 'created_at', t.created_at) as tag_obj
                        FROM post_tags pt
                        JOIN tags t ON pt.tag_id = t.id
                        WHERE pt.post_id = p.id
                    ) tags_subq
                ),
                '[]'::json
            ) as tags
        FROM posts p
        WHERE p.published = true
        GROUP BY p.id
        ORDER BY p.published_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let summaries: Vec<PostSummary> = rows
        .into_iter()
        .map(|row| {
            let tags_json: serde_json::Value = row.get("tags");
            let tags: Vec<Tag> = serde_json::from_value(tags_json).unwrap_or_default();
            let body: String = row.get("body");
            let reading_time = crate::markdown::calculate_reading_time(&body);

            PostSummary {
                id: row.get("id"),
                slug: row.get("slug"),
                title: row.get("title"),
                excerpt: row.get("excerpt"),
                published_at: row.get("published_at"),
                reading_time,
                tags,
            }
        })
        .collect();

    Ok(summaries)
}

/// Get a published post by slug
pub async fn get_post_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Post>> {
    let row: Option<PgRow> = sqlx::query(
        r#"
        SELECT
            p.*,
            COALESCE(
                (
                    SELECT json_agg(tag_obj ORDER BY (tag_obj->>'name'))
                    FROM (
                        SELECT json_build_object('id', t.id, 'name', t.name, 'color', t.color, 'created_at', t.created_at) as tag_obj
                        FROM post_tags pt
                        JOIN tags t ON pt.tag_id = t.id
                        WHERE pt.post_id = p.id
                    ) tags_subq
                ),
                '[]'::json
            ) as tags
        FROM posts p
        WHERE p.slug = $1 AND p.published = true
        "#
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let tags_json: serde_json::Value = row.get("tags");
            let tags: Vec<Tag> = serde_json::from_value(tags_json).unwrap_or_default();

            Ok(Some(Post {
                id: row.get("id"),
                slug: row.get("slug"),
                title: row.get("title"),
                excerpt: row.get("excerpt"),
                body: row.get("body"),
                published: row.get("published"),
                published_at: row.get("published_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                author_id: row.get("author_id"),
                tags,
            }))
        }
        None => Ok(None),
    }
}

/// Get any post by slug (including unpublished) - for admin operations
pub async fn get_post_by_slug_any(pool: &PgPool, slug: &str) -> Result<Option<Post>> {
    let row: Option<PgRow> = sqlx::query(
        r#"
        SELECT
            p.*,
            COALESCE(
                (
                    SELECT json_agg(tag_obj ORDER BY (tag_obj->>'name'))
                    FROM (
                        SELECT json_build_object('id', t.id, 'name', t.name, 'color', t.color, 'created_at', t.created_at) as tag_obj
                        FROM post_tags pt
                        JOIN tags t ON pt.tag_id = t.id
                        WHERE pt.post_id = p.id
                    ) tags_subq
                ),
                '[]'::json
            ) as tags
        FROM posts p
        WHERE p.slug = $1
        "#
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let tags_json: serde_json::Value = row.get("tags");
            let tags: Vec<Tag> = serde_json::from_value(tags_json).unwrap_or_default();

            Ok(Some(Post {
                id: row.get("id"),
                slug: row.get("slug"),
                title: row.get("title"),
                excerpt: row.get("excerpt"),
                body: row.get("body"),
                published: row.get("published"),
                published_at: row.get("published_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                author_id: row.get("author_id"),
                tags,
            }))
        }
        None => Ok(None),
    }
}


/// Get any post by ID (including unpublished)
pub async fn get_post_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Post>> {
    let row: Option<PgRow> = sqlx::query(
        r#"
        SELECT
            p.*,
            COALESCE(
                (
                    SELECT json_agg(tag_obj ORDER BY (tag_obj->>'name'))
                    FROM (
                        SELECT json_build_object('id', t.id, 'name', t.name, 'color', t.color, 'created_at', t.created_at) as tag_obj
                        FROM post_tags pt
                        JOIN tags t ON pt.tag_id = t.id
                        WHERE pt.post_id = p.id
                    ) tags_subq
                ),
                '[]'::json
            ) as tags
        FROM posts p
        WHERE p.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let tags_json: serde_json::Value = row.get("tags");
            let tags: Vec<Tag> = serde_json::from_value(tags_json).unwrap_or_default();

            Ok(Some(Post {
                id: row.get("id"),
                slug: row.get("slug"),
                title: row.get("title"),
                excerpt: row.get("excerpt"),
                body: row.get("body"),
                published: row.get("published"),
                published_at: row.get("published_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                author_id: row.get("author_id"),
                tags,
            }))
        }
        None => Ok(None),
    }
}

/// Create a new post
pub async fn create_post(pool: &PgPool, req: CreatePostRequest, author_id: Uuid) -> Result<Post> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let published_at = if req.published { Some(now) } else { None };

    // Start transaction
    let mut tx = pool.begin().await?;

    // Insert post
    sqlx::query(
        r#"
        INSERT INTO posts (id, slug, title, excerpt, body, published, published_at, created_at, updated_at, author_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#
    )
    .bind(id)
    .bind(&req.slug)
    .bind(&req.title)
    .bind(&req.excerpt)
    .bind(&req.body)
    .bind(req.published)
    .bind(published_at)
    .bind(now)
    .bind(now)
    .bind(author_id)
    .execute(&mut *tx)
    .await?;

    // Insert tags
    for tag_id in req.tags.iter() {
        sqlx::query("INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2)")
            .bind(id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    // Fetch the created post with tags
    get_post_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch created post"))
}

/// Update an existing post
pub async fn update_post(pool: &PgPool, id: Uuid, req: UpdatePostRequest) -> Result<Post> {
    let mut tx = pool.begin().await?;

    // Update post fields
    if let Some(slug) = &req.slug {
        sqlx::query("UPDATE posts SET slug = $1, updated_at = $2 WHERE id = $3")
            .bind(slug)
            .bind(Utc::now())
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(title) = &req.title {
        sqlx::query("UPDATE posts SET title = $1, updated_at = $2 WHERE id = $3")
            .bind(title)
            .bind(Utc::now())
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(excerpt) = &req.excerpt {
        sqlx::query("UPDATE posts SET excerpt = $1, updated_at = $2 WHERE id = $3")
            .bind(excerpt)
            .bind(Utc::now())
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(body) = &req.body {
        sqlx::query("UPDATE posts SET body = $1, updated_at = $2 WHERE id = $3")
            .bind(body)
            .bind(Utc::now())
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    // Update tags if provided
    if let Some(tag_ids) = req.tags {
        // Delete existing tags
        sqlx::query("DELETE FROM post_tags WHERE post_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // Insert new tags
        for tag_id in tag_ids {
            sqlx::query("INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2)")
                .bind(id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;

    get_post_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Post not found"))
}

/// Delete a post
pub async fn delete_post(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Publish a post
pub async fn publish_post(pool: &PgPool, id: Uuid) -> Result<Post> {
    sqlx::query(
        "UPDATE posts SET published = true, published_at = $1, updated_at = $2 WHERE id = $3",
    )
    .bind(Utc::now())
    .bind(Utc::now())
    .bind(id)
    .execute(pool)
    .await?;

    get_post_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Post not found"))
}

/// Unpublish a post
pub async fn unpublish_post(pool: &PgPool, id: Uuid) -> Result<Post> {
    sqlx::query(
        "UPDATE posts SET published = false, published_at = NULL, updated_at = $1 WHERE id = $2",
    )
    .bind(Utc::now())
    .bind(id)
    .execute(pool)
    .await?;

    get_post_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Post not found"))
}

/// List all posts (including unpublished) for admin
pub async fn list_all_posts(pool: &PgPool) -> Result<Vec<Post>> {
    let rows: Vec<PgRow> = sqlx::query(
        r#"
        SELECT
            p.*,
            COALESCE(
                (
                    SELECT json_agg(tag_obj ORDER BY (tag_obj->>'name'))
                    FROM (
                        SELECT json_build_object('id', t.id, 'name', t.name, 'color', t.color, 'created_at', t.created_at) as tag_obj
                        FROM post_tags pt
                        JOIN tags t ON pt.tag_id = t.id
                        WHERE pt.post_id = p.id
                    ) tags_subq
                ),
                '[]'::json
            ) as tags
        FROM posts p
        GROUP BY p.id
        ORDER BY p.created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let result: Vec<Post> = rows
        .into_iter()
        .map(|row| {
            let tags_json: serde_json::Value = row.get("tags");
            let tags: Vec<Tag> = serde_json::from_value(tags_json).unwrap_or_default();

            Post {
                id: row.get("id"),
                slug: row.get("slug"),
                title: row.get("title"),
                excerpt: row.get("excerpt"),
                body: row.get("body"),
                published: row.get("published"),
                published_at: row.get("published_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                author_id: row.get("author_id"),
                tags,
            }
        })
        .collect();

    Ok(result)
}

/// Get post statistics
pub async fn get_post_stats(pool: &PgPool) -> Result<serde_json::Value> {
    let row: PgRow = sqlx::query(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE published = true) as published_count,
            COUNT(*) FILTER (WHERE published = false) as draft_count,
            COUNT(*) as total_count,
            COUNT(DISTINCT author_id) as author_count,
            (SELECT COUNT(*) FROM tags) as tag_count
        FROM posts
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(json!({
        "published": row.get::<Option<i64>, _>("published_count").unwrap_or(0),
        "drafts": row.get::<Option<i64>, _>("draft_count").unwrap_or(0),
        "total": row.get::<Option<i64>, _>("total_count").unwrap_or(0),
        "authors": row.get::<Option<i64>, _>("author_count").unwrap_or(0),
        "tags": row.get::<Option<i64>, _>("tag_count").unwrap_or(0)
    }))
}

/// Search posts with full-text search
pub async fn search_posts(pool: &PgPool, query: &str) -> Result<Vec<Post>> {
    let search_pattern = format!("%{}%", query);

    let rows: Vec<PgRow> = sqlx::query(
        r#"
        SELECT
            p.*,
            COALESCE(
                (
                    SELECT json_agg(tag_obj ORDER BY (tag_obj->>'name'))
                    FROM (
                        SELECT json_build_object('id', t.id, 'name', t.name, 'color', t.color, 'created_at', t.created_at) as tag_obj
                        FROM post_tags pt
                        JOIN tags t ON pt.tag_id = t.id
                        WHERE pt.post_id = p.id
                    ) tags_subq
                ),
                '[]'::json
            ) as tags
        FROM posts p
        WHERE
            p.title ILIKE $1 OR
            p.body ILIKE $1 OR
            p.excerpt ILIKE $1 OR
            p.slug ILIKE $1
        GROUP BY p.id
        ORDER BY p.created_at DESC
        "#
    )
    .bind(search_pattern)
    .fetch_all(pool)
    .await?;

    let result: Vec<Post> = rows
        .into_iter()
        .map(|row| {
            let tags_json: serde_json::Value = row.get("tags");
            let tags: Vec<Tag> = serde_json::from_value(tags_json).unwrap_or_default();

            Post {
                id: row.get("id"),
                slug: row.get("slug"),
                title: row.get("title"),
                excerpt: row.get("excerpt"),
                body: row.get("body"),
                published: row.get("published"),
                published_at: row.get("published_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                author_id: row.get("author_id"),
                tags,
            }
        })
        .collect();

    Ok(result)
}

/// List all tags
pub async fn list_tags(pool: &PgPool) -> Result<Vec<Tag>> {
    let rows = sqlx::query("SELECT id, name, color, created_at FROM tags ORDER BY name")
        .fetch_all(pool)
        .await?;

    let tags: Vec<Tag> = rows
        .into_iter()
        .map(|row| Tag {
            id: row.get("id"),
            name: row.get("name"),
            color: row.get("color"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(tags)
}

/// Create a new tag
pub async fn create_tag(pool: &PgPool, req: CreateTagRequest) -> Result<Tag> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let row = sqlx::query(
        "INSERT INTO tags (id, name, color, created_at) VALUES ($1, $2, $3, $4) RETURNING id, name, color, created_at"
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.color)
    .bind(now)
    .fetch_one(pool)
    .await?;

    let tag = Tag {
        id: row.get("id"),
        name: row.get("name"),
        color: row.get("color"),
        created_at: row.get("created_at"),
    };

    Ok(tag)
}

/// Update a tag
pub async fn update_tag(pool: &PgPool, id: Uuid, req: CreateTagRequest) -> Result<Tag> {
    let row = sqlx::query(
        "UPDATE tags SET name = $1, color = $2 WHERE id = $3 RETURNING id, name, color, created_at",
    )
    .bind(&req.name)
    .bind(&req.color)
    .bind(id)
    .fetch_one(pool)
    .await?;

    let tag = Tag {
        id: row.get("id"),
        name: row.get("name"),
        color: row.get("color"),
        created_at: row.get("created_at"),
    };

    Ok(tag)
}

/// Delete a tag
pub async fn delete_tag(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get related posts by tags
pub async fn get_related_posts(
    pool: &PgPool,
    post_id: Uuid,
    limit: i64,
) -> Result<Vec<PostSummary>> {
    let rows: Vec<PgRow> = sqlx::query(
        r#"
        SELECT
            p.id,
            p.slug,
            p.title,
            p.excerpt,
            p.body,
            p.published_at,
            COUNT(DISTINCT pt2.tag_id) as common_tags,
            COALESCE(
                (
                    SELECT json_agg(tag_obj ORDER BY (tag_obj->>'name'))
                    FROM (
                        SELECT json_build_object('id', t.id, 'name', t.name, 'color', t.color, 'created_at', t.created_at) as tag_obj
                        FROM post_tags pt
                        JOIN tags t ON pt.tag_id = t.id
                        WHERE pt.post_id = p.id
                    ) tags_subq
                ),
                '[]'::json
            ) as tags
        FROM posts p
        INNER JOIN post_tags pt1 ON pt1.post_id = $1
        INNER JOIN post_tags pt2 ON pt2.tag_id = pt1.tag_id AND pt2.post_id = p.id
        WHERE p.id != $1 AND p.published = true
        GROUP BY p.id
        ORDER BY common_tags DESC, p.published_at DESC
        LIMIT $2
        "#
    )
    .bind(post_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let summaries: Vec<PostSummary> = rows
        .into_iter()
        .map(|row| {
            let tags_json: serde_json::Value = row.get("tags");
            let tags: Vec<Tag> = serde_json::from_value(tags_json).unwrap_or_default();
            let body: String = row.get("body");
            let reading_time = crate::markdown::calculate_reading_time(&body);

            PostSummary {
                id: row.get("id"),
                slug: row.get("slug"),
                title: row.get("title"),
                excerpt: row.get("excerpt"),
                published_at: row.get("published_at"),
                reading_time,
                tags,
            }
        })
        .collect();

    Ok(summaries)
}

/// Get published posts by tag name
pub async fn get_posts_by_tag(pool: &PgPool, tag_name: &str) -> Result<Vec<PostSummary>> {
    let rows = sqlx::query(
        r#"
        SELECT
            p.id,
            p.slug,
            p.title,
            p.excerpt,
            p.body,
            p.published_at,
            COALESCE(
                (
                    SELECT json_agg(tag_obj ORDER BY (tag_obj->>'name'))
                    FROM (
                        SELECT json_build_object('id', t.id, 'name', t.name, 'color', t.color, 'created_at', t.created_at) as tag_obj
                        FROM post_tags pt
                        JOIN tags t ON pt.tag_id = t.id
                        WHERE pt.post_id = p.id
                    ) tags_subq
                ),
                '[]'::json
            ) as tags
        FROM posts p
        WHERE p.published = true
            AND p.id IN (
                SELECT pt2.post_id
                FROM post_tags pt2
                JOIN tags t2 ON pt2.tag_id = t2.id
                WHERE t2.name = $1
            )
        GROUP BY p.id
        ORDER BY p.published_at DESC
        "#
    )
    .bind(tag_name)
    .fetch_all(pool)
    .await?;

    let summaries: Vec<PostSummary> = rows
        .into_iter()
        .map(|row| {
            let tags_json: serde_json::Value = row.get("tags");
            let tags: Vec<Tag> = serde_json::from_value(tags_json).unwrap_or_default();
            let body: String = row.get("body");
            let reading_time = crate::markdown::calculate_reading_time(&body);

            PostSummary {
                id: row.get("id"),
                slug: row.get("slug"),
                title: row.get("title"),
                excerpt: row.get("excerpt"),
                published_at: row.get("published_at"),
                reading_time,
                tags,
            }
        })
        .collect();

    Ok(summaries)
}

/// Get tag statistics
pub async fn get_tag_stats(pool: &PgPool) -> Result<Vec<crate::handlers::tags::TagStats>> {
    let rows = sqlx::query(
        r#"
        SELECT
            t.id,
            t.name,
            t.color,
            t.created_at,
            COUNT(pt.post_id) as post_count
        FROM tags t
        LEFT JOIN post_tags pt ON t.id = pt.tag_id
        LEFT JOIN posts p ON pt.post_id = p.id AND p.published = true
        GROUP BY t.id, t.name, t.color, t.created_at
        ORDER BY post_count DESC, t.name
        "#
    )
    .fetch_all(pool)
    .await?;

    let stats: Vec<crate::handlers::tags::TagStats> = rows
        .into_iter()
        .map(|row| {
            let tag = Tag {
                id: row.get("id"),
                name: row.get("name"),
                color: row.get("color"),
                created_at: row.get("created_at"),
            };
            let post_count: i64 = row.get("post_count");
            crate::handlers::tags::TagStats {
                tag,
                post_count: post_count as usize,
            }
        })
        .collect();

    Ok(stats)
}
