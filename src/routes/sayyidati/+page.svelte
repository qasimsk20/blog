<script lang="ts">
    import {
        formatPublishedDate,
        formatReadingTime,
        getAllPosts,
        type BlogPost,
        type BlogTag,
        type CatppuccinColor,
    } from "$lib/data/posts";

    type Draft = {
        title: string;
        slug: string;
        excerpt: string;
        body: string;
        tags: BlogTag[];
    };

    type TagInput = {
        name: string;
        color: CatppuccinColor;
    };

    const CATPPUCCIN_COLORS: CatppuccinColor[] = [
        "rosewater",
        "flamingo",
        "pink",
        "mauve",
        "red",
        "maroon",
        "peach",
        "yellow",
        "green",
        "teal",
        "sky",
        "sapphire",
        "blue",
        "lavender",
    ];

    const initialPosts = getAllPosts();

    let posts = $state<BlogPost[]>(structuredClone(initialPosts));
    let selectedSlug = $state<string | null>(posts[0]?.slug ?? null);
    let search = $state("");
    let showCreatePanel = $state(false);
    let draft = $state<Draft>({
        title: "",
        slug: "",
        excerpt: "",
        body: "",
        tags: [],
    });
    let newTag = $state<TagInput>({ name: "", color: "lavender" });
    let toast = $state<{
        message: string;
        tone: "neutral" | "success" | "warn";
    } | null>(null);

    let filteredPosts = $derived(() => {
        const q = search.trim().toLowerCase();
        if (!q) {
            return posts;
        }
        return posts.filter((post) => {
            const tagNames = post.tags.map((t) => t.name);
            const haystack = [post.title, post.slug, post.excerpt, ...tagNames]
                .join(" ")
                .toLowerCase();
            return haystack.includes(q);
        });
    });

    let selectedPost = $derived(() => {
        if (!selectedSlug) {
            return null;
        }
        return posts.find((post) => post.slug === selectedSlug) ?? null;
    });

    function selectPost(slug: string) {
        selectedSlug = slug;
        showCreatePanel = false;
    }

    function resetDraft(overrides: Partial<Draft> = {}) {
        draft = {
            title: "",
            slug: "",
            excerpt: "",
            body: "",
            tags: [],
            ...overrides,
        };
        newTag = { name: "", color: "lavender" };
    }

    function openCreatePanel() {
        resetDraft();
        showCreatePanel = true;
        selectedSlug = null;
    }

    function notify(
        message: string,
        tone: "neutral" | "success" | "warn" = "neutral",
    ) {
        toast = { message, tone };
        setTimeout(() => {
            toast = null;
        }, 2400);
    }

    function addTagToDraft() {
        const trimmedName = newTag.name.trim();
        if (!trimmedName) {
            notify("tag name required", "warn");
            return;
        }
        if (draft.tags.some((t) => t.name === trimmedName)) {
            notify("tag already exists", "warn");
            return;
        }
        draft.tags = [
            ...draft.tags,
            { name: trimmedName, color: newTag.color },
        ];
        newTag = { name: "", color: "lavender" };
    }

    function removeTagFromDraft(tagName: string) {
        draft.tags = draft.tags.filter((t) => t.name !== tagName);
    }

    function createPost() {
        const trimmed = {
            title: draft.title.trim(),
            slug: draft.slug.trim(),
            excerpt: draft.excerpt.trim(),
            body: draft.body.trim(),
        };

        if (!trimmed.title || !trimmed.slug || !trimmed.body) {
            notify("title, slug, and body are required", "warn");
            return;
        }

        if (posts.some((post) => post.slug === trimmed.slug)) {
            notify("slug already in use", "warn");
            return;
        }

        const created: BlogPost = {
            title: trimmed.title,
            slug: trimmed.slug,
            excerpt: trimmed.excerpt || trimmed.body.slice(0, 160),
            body: trimmed.body,
            publishedAt: new Date().toISOString(),
            tags: draft.tags,
            readingTime: formatReadingTime(trimmed.body),
        };

        posts = [created, ...posts];
        selectedSlug = created.slug;
        showCreatePanel = false;
        notify("post created (local only)", "success");
    }

    function deletePost(slug: string) {
        posts = posts.filter((post) => post.slug !== slug);
        if (selectedSlug === slug) {
            selectedSlug = posts[0]?.slug ?? null;
        }
        notify("post removed from console state", "warn");
    }

    function duplicatePost(post: BlogPost) {
        const stamp = Date.now().toString(36);
        const clone: BlogPost = {
            ...post,
            slug: `${post.slug}-${stamp}`,
            title: `${post.title} (copy)`,
            publishedAt: new Date().toISOString(),
        };
        posts = [clone, ...posts];
        selectedSlug = clone.slug;
        showCreatePanel = false;
        notify("duplicate post drafted", "success");
    }

    function loadDraft(post: BlogPost) {
        resetDraft({
            title: post.title,
            slug: post.slug,
            excerpt: post.excerpt,
            body: post.body,
            tags: [...post.tags],
        });
        showCreatePanel = true;
        selectedSlug = null;
    }

    function updatePostField(
        post: BlogPost,
        field: "title" | "excerpt" | "body",
        value: string,
    ) {
        posts = posts.map((entry) => {
            if (entry.slug !== post.slug) {
                return entry;
            }
            const updated = { ...entry };
            if (field === "body") {
                updated.body = value;
                updated.readingTime = formatReadingTime(value);
            } else if (field === "excerpt") {
                updated.excerpt = value;
            } else if (field === "title") {
                updated.title = value;
            }
            return updated;
        });
        notify("post updated locally", "success");
    }

    function addTagToPost(post: BlogPost, tag: BlogTag) {
        posts = posts.map((entry) => {
            if (entry.slug !== post.slug) {
                return entry;
            }
            if (entry.tags.some((t) => t.name === tag.name)) {
                return entry;
            }
            return { ...entry, tags: [...entry.tags, tag] };
        });
        notify("tag added", "success");
    }

    function removeTagFromPost(post: BlogPost, tagName: string) {
        posts = posts.map((entry) => {
            if (entry.slug !== post.slug) {
                return entry;
            }
            return {
                ...entry,
                tags: entry.tags.filter((t) => t.name !== tagName),
            };
        });
        notify("tag removed", "success");
    }
</script>
