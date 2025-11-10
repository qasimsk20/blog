use ammonia::{Builder, UrlRelative};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html};
use regex::Regex;
use std::collections::HashSet;

/// Obsidian-style callout types with Catppuccin colors
#[derive(Debug, Clone)]
pub struct CalloutType {
    pub name: &'static str,
    pub icon: &'static str,  // Nerd Font icons
    pub color: &'static str, // Catppuccin color
}

impl CalloutType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "note" | "info" => Self {
                name: "note",
                icon: "", // nf-fa-info_circle
                color: "blue",
            },
            "tip" | "hint" | "important" => Self {
                name: "tip",
                icon: "", // nf-fa-lightbulb_o
                color: "teal",
            },
            "warning" | "caution" | "attention" => Self {
                name: "warning",
                icon: "", // nf-fa-exclamation_triangle
                color: "yellow",
            },
            "danger" | "error" => Self {
                name: "danger",
                icon: "", // nf-fa-fire
                color: "red",
            },
            "success" | "check" | "done" => Self {
                name: "success",
                icon: "", // nf-fa-check_circle
                color: "green",
            },
            "question" | "help" | "faq" => Self {
                name: "question",
                icon: "", // nf-fa-question_circle
                color: "mauve",
            },
            "example" => Self {
                name: "example",
                icon: "", // nf-fa-file_text_o
                color: "lavender",
            },
            "quote" | "cite" => Self {
                name: "quote",
                icon: "", // nf-fa-quote_left
                color: "flamingo",
            },
            "bug" => Self {
                name: "bug",
                icon: "", // nf-fa-bug
                color: "maroon",
            },
            "abstract" | "summary" | "tldr" => Self {
                name: "abstract",
                icon: "", // nf-fa-file_text
                color: "sky",
            },
            "code" | "snippet" => Self {
                name: "code",
                icon: "", // nf-fa-code
                color: "peach",
            },
            "todo" | "task" => Self {
                name: "todo",
                icon: "", // nf-fa-check_square_o
                color: "sapphire",
            },
            _ => Self {
                name: "note",
                icon: "", // nf-oct-pin
                color: "surface2",
            },
        }
    }
}

/// Process Obsidian-style markdown into HTML
pub fn render_obsidian_markdown(content: &str) -> String {
    // Pre-process Obsidian-specific syntax
    let processed = preprocess_obsidian_syntax(content);

    // Configure parser options
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(&processed, options);

    // Transform events for syntax highlighting and custom rendering
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut events = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                if let CodeBlockKind::Fenced(lang) = kind {
                    code_lang = lang.to_string();
                }
                events.push(Event::Html(
                    format!(
                        r#"<div class="code-block" data-lang="{}">
                            <div class="code-header">
                                <span class="code-lang">{}</span>
                                <button class="code-copy" onclick="copyCode(this)" aria-label="Copy code">
                                    <span class="copy-icon"></span>
                                </button>
                            </div>
                            <pre><code class="language-{}">"#,
                        code_lang,
                        if code_lang.is_empty() { "text" } else { &code_lang },
                        code_lang
                    ).into()
                ));
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                code_lang.clear();
                events.push(Event::Html("</code></pre></div>".into()));
            }
            Event::Text(text) if in_code_block => {
                events.push(Event::Html(escape_html(&text).into()));
            }
            Event::Code(code) => {
                events.push(Event::Html(
                    format!(r#"<code class="inline-code">{}</code>"#, escape_html(&code)).into(),
                ));
            }
            _ => events.push(event),
        }
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, events.into_iter());

    // Post-process for callouts and other Obsidian features
    let html_output = postprocess_callouts(&html_output);
    let html_output = postprocess_highlights(&html_output);
    let html_output = postprocess_mermaid_diagrams(&html_output);

    // Sanitize HTML while preserving our custom elements
    sanitize_html(&html_output)
}

/// Pre-process Obsidian-specific syntax before parsing
fn preprocess_obsidian_syntax(content: &str) -> String {
    let mut processed = content.to_string();

    // Process wiki-links [[Page]] or [[Page|Display Text]]
    let wiki_link_re = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
    processed = wiki_link_re
        .replace_all(&processed, |caps: &regex::Captures| {
            let link = &caps[1];
            let display = caps.get(2).map(|m| m.as_str()).unwrap_or(link);
            let slug = slugify(link);
            format!(
                r#"<a href="/blogs/{}" class="wiki-link" data-page="{}"><span class="link-icon"></span> {}</a>"#,
                slug, link, display
            )
        })
        .to_string();

    // Process tags #tag
    let tag_re = Regex::new(r"(?:^|\s)#([a-zA-Z][a-zA-Z0-9_-]*)").unwrap();
    processed = tag_re
        .replace_all(&processed, |caps: &regex::Captures| {
            let tag = &caps[1];
            format!(
                r#" <span class="obsidian-tag" data-tag="{}"><span class="tag-icon"></span>{}</span>"#,
                tag, tag
            )
        })
        .to_string();

    // Process block IDs ^block-id
    let block_id_re = Regex::new(r"\^([a-zA-Z0-9-]+)$").unwrap();
    processed = block_id_re
        .replace_all(&processed, |caps: &regex::Captures| {
            let block_id = &caps[1];
            format!(
                r#"<span class="block-ref" id="block-{}" data-block-id="{}"></span>"#,
                block_id, block_id
            )
        })
        .to_string();

    // Process embedded content ![[Image]] or ![[Page]]
    let embed_re = Regex::new(r"!\[\[([^\]]+)\]\]").unwrap();
    processed = embed_re
        .replace_all(&processed, |caps: &regex::Captures| {
            let resource = &caps[1];
            if is_image(resource) {
                format!(
                    r#"<img src="/api/assets/{}" alt="{}" class="obsidian-embed-image" loading="lazy" />"#,
                    slugify(resource),
                    resource
                )
            } else {
                format!(
                    r#"<div class="obsidian-embed" data-page="{}"><span class="embed-icon"></span> {}</div>"#,
                    resource, resource
                )
            }
        })
        .to_string();

    processed
}

/// Process callouts in the HTML output
fn postprocess_callouts(html: &str) -> String {
    // Match blockquotes that start with [!type]
    let callout_re =
        Regex::new(r#"<blockquote>\s*<p>\[!([^\]]+)\](?:\s+(.+?))?</p>([\s\S]*?)</blockquote>"#)
            .unwrap();

    callout_re
        .replace_all(html, |caps: &regex::Captures| {
            let callout_type = CalloutType::from_str(&caps[1]);
            let title = caps.get(2).map(|m| m.as_str()).unwrap_or(&caps[1]);
            let content = &caps[3];

            format!(
                r#"<div class="callout callout-{}" data-callout-type="{}">
                <div class="callout-header">
                    <span class="callout-icon">{}</span>
                    <span class="callout-title">{}</span>
                    <button class="callout-fold" onclick="toggleCallout(this)" aria-label="Toggle callout">
                        <span class="fold-icon"></span>
                    </button>
                </div>
                <div class="callout-content">{}</div>
            </div>"#,
                callout_type.color, callout_type.name, callout_type.icon, title, content
            )
        })
        .to_string()
}

/// Process highlighting syntax ==text==
fn postprocess_highlights(html: &str) -> String {
    let highlight_re = Regex::new(r"==(.*?)==").unwrap();
    highlight_re
        .replace_all(html, r#"<mark class="obsidian-highlight">$1</mark>"#)
        .to_string()
}

/// Process Mermaid diagrams
fn postprocess_mermaid_diagrams(html: &str) -> String {
    let mermaid_re =
        Regex::new(r#"<pre><code class="language-mermaid">([\s\S]*?)</code></pre>"#).unwrap();
    mermaid_re
        .replace_all(html, |caps: &regex::Captures| {
            let diagram = &caps[1];
            format!(
                r#"<div class="mermaid-diagram" data-diagram="{}">
                    <div class="mermaid-loading"><span class="loading-icon"></span> Rendering diagram...</div>
                    <div class="mermaid-content" style="display:none;">{}</div>
                </div>"#,
                escape_html(diagram), diagram
            )
        })
        .to_string()
}

/// Calculate reading time from content
pub fn calculate_reading_time(content: &str) -> String {
    let word_count = content.split_whitespace().count();
    let reading_time = (word_count as f64 / 200.0).ceil() as u32; // 200 words per minute

    if reading_time <= 1 {
        "1 min read".to_string()
    } else {
        format!("{} min read", reading_time)
    }
}

/// Extract plain text excerpt from markdown
pub fn extract_excerpt(content: &str, max_length: usize) -> String {
    // Remove Obsidian-specific syntax first
    let mut plain = content.to_string();

    // Remove wiki links
    let wiki_link_re = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
    plain = wiki_link_re.replace_all(&plain, "$2").to_string();

    // Remove tags
    let tag_re = Regex::new(r"#[a-zA-Z][a-zA-Z0-9_-]*").unwrap();
    plain = tag_re.replace_all(&plain, "").to_string();

    // Remove highlights
    let highlight_re = Regex::new(r"==(.*?)==").unwrap();
    plain = highlight_re.replace_all(&plain, "$1").to_string();

    // Parse as markdown to get plain text
    let parser = Parser::new(&plain);
    let mut plain_text = String::new();

    for event in parser {
        match event {
            Event::Text(text) => plain_text.push_str(&text),
            Event::SoftBreak | Event::HardBreak => plain_text.push(' '),
            _ => {}
        }
    }

    // Truncate to max length at word boundary
    if plain_text.len() <= max_length {
        plain_text
    } else {
        let mut excerpt = plain_text.chars().take(max_length).collect::<String>();

        // Find last space to avoid cutting words
        if let Some(last_space) = excerpt.rfind(' ') {
            excerpt.truncate(last_space);
        }

        format!("{}...", excerpt.trim())
    }
}

/// Extract all tags from markdown content
pub fn extract_tags(content: &str) -> Vec<String> {
    let tag_re = Regex::new(r"#([a-zA-Z][a-zA-Z0-9_-]*)").unwrap();
    let mut tags = HashSet::new();

    for cap in tag_re.captures_iter(content) {
        tags.insert(cap[1].to_string());
    }

    tags.into_iter().collect()
}

/// Extract all wiki-links from markdown content
pub fn extract_links(content: &str) -> Vec<String> {
    let wiki_link_re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
    let mut links = HashSet::new();

    for cap in wiki_link_re.captures_iter(content) {
        links.insert(cap[1].to_string());
    }

    links.into_iter().collect()
}

/// Strip the first heading (h1) from markdown content
pub fn strip_first_heading(content: &str) -> String {
    if content.starts_with("# ") {
        if let Some(pos) = content.find("\n\n") {
            return content[pos + 2..].to_string();
        }
    }
    content.to_string()
}

/// Convert a title to a URL slug
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Check if a resource name is an image
fn is_image(resource: &str) -> bool {
    let lower = resource.to_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
        || lower.ends_with(".svg")
        || lower.ends_with(".avif")
}

/// Escape HTML special characters
fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Sanitize HTML while preserving Obsidian elements
fn sanitize_html(html: &str) -> String {
    let mut builder = Builder::default();

    // Allow our custom elements and attributes
    use std::collections::HashMap;
    use std::collections::HashSet;

    let mut tag_attributes = HashMap::new();
    tag_attributes.insert("a", HashSet::from(["data-page"]));
    tag_attributes.insert("span", HashSet::from(["data-tag", "data-block-id", "id"]));
    tag_attributes.insert("div", HashSet::from(["data-page", "data-callout-type", "data-lang", "data-diagram"]));
    tag_attributes.insert("button", HashSet::from(["onclick", "aria-label"]));
    tag_attributes.insert("img", HashSet::from(["src", "alt", "loading"]));

    let mut allowed_classes = HashMap::new();
    allowed_classes.insert("a", HashSet::from(["wiki-link"]));
    allowed_classes.insert("span", HashSet::from([
        "inline-code", "bold", "italic", "strikethrough", "highlight",
        "fold-icon", "loading-icon"
    ]));
    let mut div_classes = HashSet::from([
        "obsidian-embed", "callout", "callout-header", "callout-content",
        "code-block", "code-header", "mermaid-diagram", "mermaid-loading",
        "mermaid-content"
    ]);

    // Add callout color classes
    let callout_class_names = [
        "callout-rosewater", "callout-flamingo", "callout-pink", "callout-mauve", "callout-red",
        "callout-maroon", "callout-peach", "callout-yellow", "callout-green", "callout-teal",
        "callout-sky", "callout-sapphire", "callout-blue", "callout-lavender", "callout-surface2"
    ];
    for class_name in callout_class_names {
        div_classes.insert(class_name);
    }

    allowed_classes.insert("div", div_classes);
    allowed_classes.insert("button", HashSet::from(["callout-fold", "code-copy"]));
    allowed_classes.insert("code", HashSet::from(["inline-code"]));
    allowed_classes.insert("mark", HashSet::from(["obsidian-highlight"]));
    allowed_classes.insert("img", HashSet::from(["obsidian-embed-image"]));

    builder
        .link_rel(Some("noopener noreferrer"))
        .url_relative(UrlRelative::PassThrough)
        .tag_attributes(tag_attributes)
        .allowed_classes(allowed_classes);

    // Callout color classes are now added to allowed_classes above

    builder.clean(html).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wiki_links() {
        let content = "Check out [[My Page]] and [[Other Page|this link]]";
        let processed = preprocess_obsidian_syntax(content);
        assert!(processed.contains(r#"href="/blogs/my-page""#));
        assert!(processed.contains("this link</a>"));
    }

    #[test]
    fn test_tags() {
        let content = "This is #important and #urgent";
        let tags = extract_tags(content);
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"important".to_string()));
        assert!(tags.contains(&"urgent".to_string()));
    }

    #[test]
    fn test_callout_types() {
        let note = CalloutType::from_str("note");
        assert_eq!(note.color, "blue");

        let warning = CalloutType::from_str("warning");
        assert_eq!(warning.color, "yellow");
    }

    #[test]
    fn test_reading_time() {
        assert_eq!(calculate_reading_time("hello world"), "1 min read");
        assert_eq!(calculate_reading_time(&"word ".repeat(400)), "2 min read");
    }
}
