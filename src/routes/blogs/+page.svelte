<script lang="ts">
    import type { BlogPostSummary, BlogTag } from "$lib/data/posts";

    const { data } = $props<{
        data: {
            posts: BlogPostSummary[];
            tags: BlogTag[];
            meta: {
                total: number;
            };
        };
    }>();

    let query = $state("");
    let activeTags = $state<Set<string>>(new Set());

    let filteredPosts = $state<BlogPostSummary[]>(data.posts);

    $effect(() => {
        const q = query.trim().toLowerCase();
        const selected = Array.from(activeTags).map((t) => t.toLowerCase());

        filteredPosts = data.posts.filter((post: BlogPostSummary) => {
            const matchesTags = selected.length
                ? post.tags.some((candidate: BlogTag) =>
                      selected.includes(candidate.name.toLowerCase()),
                  )
                : true;

            if (!q) {
                return matchesTags;
            }

            const haystack = [
                post.title,
                post.excerpt,
                post.publishedLabel,
                post.readingTime,
            ]
                .join(" ")
                .toLowerCase();

            return matchesTags && haystack.includes(q);
        });
    });

    let visibleCount = $derived(filteredPosts.length);
    let hasInitialPosts = $derived(data.posts.length > 0);
    let hasFilters = $derived(query.trim().length > 0 || activeTags.size > 0);

    function toggleTag(tag: string) {
        const next = new Set(activeTags);
        if (next.has(tag)) next.delete(tag);
        else next.add(tag);
        activeTags = next;
    }

    function clearFilters() {
        query = "";
        activeTags = new Set();
    }
</script>

<div class="page">
    <section class="hero">
        <h1 class="title">
            <span class="title-qasim">qasim</span><span class="title-sk"
                >sk20</span
            ><span class="title-suffix">/blogs</span>
        </h1>
        <p class="tagline">code & philosophy</p>
    </section>

    <section class="toolbar" aria-label="Filter blog posts">
        <div class="search">
            <label class="search-label" for="blog-search">search</label>
            <input
                id="blog-search"
                type="search"
                placeholder="find a post..."
                bind:value={query}
            />
        </div>

        <div class="tags" aria-label="Filter by tag">
            <button
                type="button"
                class:tag-active={activeTags.size === 0}
                onclick={clearFilters}
                aria-pressed={activeTags.size === 0}
                data-badge
            >
                all
            </button>
            {#each data.tags as tag}
                <button
                    type="button"
                    class:tag-active={activeTags.has(tag.name)}
                    onclick={() => toggleTag(tag.name)}
                    aria-pressed={activeTags.has(tag.name)}
                    data-badge
                    data-variant={tag.color}
                >
                    {tag.name}
                </button>
            {/each}
        </div>

        {#if hasFilters}
            <button type="button" class="clear-filters" onclick={clearFilters}>
                clear filters
            </button>
        {/if}
    </section>

    <section class="content" aria-live="polite">
        {#if !hasInitialPosts}
            <p class="empty">no posts yet</p>
        {:else if visibleCount === 0}
            <div class="empty">
                <p>no matches found</p>
                <p class="empty-detail">
                    tweak the search or choose another tag
                </p>
            </div>
        {:else}
            <ul class="post-list">
                {#each filteredPosts as post (post.slug)}
                    <li class="post-item">
                        <a
                            class="post-title"
                            href={`/blogs/${post.slug}`}
                            rel="bookmark"
                        >
                            {post.title}
                        </a>
                        <p class="post-excerpt">{post.excerpt}</p>
                        <div class="post-meta">
                            <span>{post.publishedLabel}</span>
                            <span aria-hidden="true">·</span>
                            <span>{post.readingTime}</span>
                        </div>
                        {#if post.tags.length > 0}
                            <div class="post-tags" aria-label="tags">
                                {#each post.tags as tag}
                                    <span data-badge data-variant={tag.color}>
                                        {tag.name}
                                    </span>
                                {/each}
                            </div>
                        {/if}
                    </li>
                {/each}
            </ul>
        {/if}
    </section>

    <footer class="footer">
        <p>
            Built with <span class="footer-svelte">Svelte</span>,
            <span class="footer-rust">Rust</span> and
            <a
                class="footer-webtui"
                href="https://x.com/webtui"
                rel="noreferrer noopener"
                ><span class="footer-web">WEB</span><span class="footer-tui"
                    >TUI</span
                ></a
            >
            by
            <a
                class="footer-iron"
                href="https://x.com/IroncladDev"
                rel="noreferrer noopener">IronClad</a
            >
        </p>
    </footer>
</div>

<style>
    .page {
        max-width: 1100px;
        margin: 0 auto;
        padding: 4rem 1.5rem 3rem;
        display: flex;
        flex-direction: column;
        gap: 3rem;
    }

    .hero {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        gap: 0.75rem;
    }

    .title {
        margin: 0;
        /*font-size: clamp(2.4rem, 6.4vw, 3.6rem);*/
        font-size: 3rem;
        font-weight: 600;
        letter-spacing: 0.02em;
    }

    .title-qasim {
        color: var(--teal);
    }

    .title-sk {
        color: var(--lavender);
    }

    .title-suffix {
        color: var(--text);
    }

    .tagline {
        margin: 0;
        color: var(--subtext0);
        letter-spacing: 0.25ch;
        text-transform: lowercase;
    }

    .toolbar {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .search {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        align-items: stretch;
    }

    .search-label {
        font-size: 0.85rem;
        text-transform: uppercase;
        letter-spacing: 0.35ch;
        color: var(--subtext1);
    }

    .search input {
        width: 100%;
        background: transparent;
        border: none;
        border-bottom: 2px solid var(--surface1);
        padding: 0.85rem 0;
        color: var(--text);
        font-family: inherit;
        font-size: 1rem;
        letter-spacing: 0.18ch;
        text-transform: lowercase;
        transition: border-color 0.2s ease;
    }

    .search input::placeholder {
        color: var(--subtext1);
    }

    .search input:focus-visible {
        outline: none;
        border-bottom-color: var(--lavender);
    }

    .tags {
        display: flex;
        flex-wrap: wrap;
        gap: 0.75rem;
    }

    /* WebTUI-like badges without invalid attributes */
    .tags [data-badge],
    .post-tags [data-badge] {
        display: inline-flex;
        align-items: center;
        gap: 0.25rem;
        padding: 0.35rem 0.75rem;
        border: 1px solid color-mix(in srgb, var(--surface1) 70%, transparent);
        background: color-mix(in srgb, var(--surface0) 45%, transparent);
        opacity: 0.65;
        filter: saturate(0.6) brightness(0.9);
        text-transform: lowercase;
        letter-spacing: 0.08ch;
        font-size: 0.9rem;
        font-family: inherit;
        appearance: none;
        color: inherit;
        box-shadow: none;
        transition:
            opacity 0.2s ease,
            filter 0.2s ease,
            border-color 0.2s ease,
            box-shadow 0.2s ease;
        /* match post tag pills */
        border-radius: 0 !important;
    }

    /* variants (matte catppuccin) */
    [data-badge][data-variant="lavender"] {
        background: var(--lavender);
        border-color: var(--lavender);
        color: var(--base);
    }
    [data-badge][data-variant="peach"] {
        background: var(--peach);
        border-color: var(--peach);
        color: var(--base);
    }
    [data-badge][data-variant="pink"] {
        background: var(--pink);
        border-color: var(--pink);
        color: var(--base);
    }
    [data-badge][data-variant="mauve"] {
        background: var(--mauve);
        border-color: var(--mauve);
        color: var(--base);
    }
    [data-badge][data-variant="maroon"] {
        background: var(--maroon);
        border-color: var(--maroon);
        color: var(--base);
    }
    [data-badge][data-variant="red"] {
        background: var(--red);
        border-color: var(--red);
        color: var(--base);
    }
    [data-badge][data-variant="rosewater"] {
        background: color-mix(in srgb, var(--rosewater) 18%, transparent);
        border-color: color-mix(in srgb, var(--rosewater) 40%, transparent);
        color: var(--rosewater);
    }
    [data-badge][data-variant="blue"] {
        background: var(--blue);
        border-color: var(--blue);
        color: var(--base);
    }
    [data-badge][data-variant="green"] {
        background: var(--green);
        border-color: var(--green);
        color: var(--base);
    }
    [data-badge][data-variant="sky"] {
        background: var(--sky);
        border-color: var(--sky);
        color: var(--base);
    }

    .tags [data-badge] {
        cursor: pointer;
    }

    .tags [data-badge]:hover {
        opacity: 0.85;
        filter: saturate(0.85) brightness(1);
    }

    .tags [data-badge]:focus-visible {
        outline: none;
        opacity: 1;
        filter: none;
        border-color: var(--lavender);
        box-shadow: 0 0 0 2px
            color-mix(in srgb, var(--lavender) 60%, transparent);
    }

    .tags [data-badge].tag-active {
        opacity: 1;
        filter: none;
        border-color: var(--lavender);
        box-shadow: 0 0 0 1px
            color-mix(in srgb, var(--lavender) 55%, transparent);
        background: color-mix(in srgb, var(--surface0) 65%, transparent);
    }

    .clear-filters {
        align-self: flex-start;
        padding: 0.35rem 1rem;
        border: 1px solid var(--surface1);
        background: transparent;
        color: var(--subtext1);
        font-size: 0.85rem;
        text-transform: lowercase;
        letter-spacing: 0.18ch;
        cursor: pointer;
        transition:
            border-color 0.2s ease,
            color 0.2s ease;
    }

    .clear-filters:hover {
        border-color: var(--peach);
        color: var(--peach);
    }

    .content {
        max-width: 900px;
        margin: 0 auto;
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
        min-height: 20rem;
    }

    .empty {
        margin: 0;
        color: var(--subtext0);
        text-transform: lowercase;
        letter-spacing: 0.25ch;
        text-align: center;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .empty-detail {
        margin: 0;
        font-size: 0.9rem;
        color: var(--subtext1);
    }

    .post-list {
        margin: 0;
        padding: 0;
        list-style: none;
        display: grid;
        gap: 2rem;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
        grid-auto-rows: minmax(0, 1fr);
        align-items: stretch;
    }

    .post-item {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        padding: 1.85rem 1.6rem;
        background: color-mix(in srgb, var(--surface0) 55%, transparent);
        border: 1px solid color-mix(in srgb, var(--surface1) 80%, transparent);
        box-shadow: 0 0 0 1px
            color-mix(in srgb, var(--surface1) 45%, transparent);
        height: 100%;
        transition:
            transform 0.2s ease,
            border-color 0.2s ease,
            box-shadow 0.2s ease,
            background 0.2s ease;
    }

    .post-item:hover,
    .post-item:focus-within {
        transform: translateY(-2px);
        border-color: var(--lavender);
        box-shadow: 0 0 0 1px var(--lavender);
        background: color-mix(in srgb, var(--surface0) 70%, transparent);
    }

    .post-title {
        font-size: clamp(1.75rem, 3vw, 2.2rem);
        color: var(--text);
        text-decoration: none;
        letter-spacing: 0.02em;
    }

    .post-title:hover {
        text-decoration: underline;
    }

    .post-title:focus-visible {
        outline: 2px solid var(--lavender);
        outline-offset: 6px;
        text-decoration: underline;
    }

    .post-excerpt {
        margin: 0;
        color: var(--subtext0);
        line-height: 1.6;
    }

    .post-meta {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        color: var(--subtext1);
        text-transform: lowercase;
        border-top: 1px solid var(--surface1);
        padding-top: 0.85rem;
        margin-top: 0.6rem;
    }

    .post-tags {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-top: 0.85rem;
        padding-top: 0.85rem;
        border-top: 1px dashed
            color-mix(in srgb, var(--surface1) 70%, transparent);
    }

    .post-tags [data-badge] {
        cursor: default;
        transition: none;
        opacity: 1;
        filter: none;
        border-color: color-mix(in srgb, var(--surface1) 60%, transparent);
        background: color-mix(in srgb, var(--surface0) 65%, transparent);
    }

    .footer {
        border-top: 1px solid var(--surface1);
        padding-top: 1.5rem;
        text-align: center;
        color: var(--subtext0);
        letter-spacing: 0.2ch;
        font-size: 0.95rem;
    }

    .footer p {
        margin: 0;
    }

    .footer-svelte {
        color: var(--peach);
    }

    .footer-rust {
        color: var(--peach);
    }

    .footer-webtui,
    .footer-iron {
        text-decoration: none;
        transition: color 0.2s ease;
    }

    .footer-webtui:hover,
    .footer-iron:hover {
        text-decoration: underline;
    }

    .footer-web {
        color: var(--green);
    }

    .footer-tui {
        color: var(--lavender);
    }

    .footer-iron {
        color: var(--lavender);
    }

    @media (max-width: 640px) {
        .page {
            padding: 3rem 1.25rem 2.5rem;
            gap: 2.5rem;
        }

        .toolbar {
            gap: 1.25rem;
        }

        .tags {
            gap: 0.5rem;
        }

        .post-item {
            padding-bottom: 2rem;
        }
    }
</style>
