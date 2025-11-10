<script lang="ts">
    import type { BlogPostSummary } from "$lib/data/posts";

    interface PostDetail extends BlogPostSummary {
        body: string;
        publishedAt: string;
    }

    const { data } = $props<{
        data: {
            post: PostDetail & { publishedLabel: string };
            adjacent: {
                previous: BlogPostSummary | null;
                next: BlogPostSummary | null;
            };
            related: BlogPostSummary[];
            meta: {
                locale: string;
            };
        };
    }>();

    import MarkdownIt from "markdown-it";
    import hljs from "highlight.js";

    const baseMarkdown = new MarkdownIt();

    const md = new MarkdownIt({
        html: false,
        linkify: true,
        typographer: true,
        highlight: (str: string, lang: string): string => {
            if (lang && hljs.getLanguage(lang)) {
                try {
                    return `<pre><code class="hljs">${hljs.highlight(str, { language: lang }).value}</code></pre>`;
                } catch {}
            }
            return `<pre><code class="hljs">${baseMarkdown.utils.escapeHtml(str)}</code></pre>`;
        },
    });

    const html = $derived(md.render(data.post.body));

    const hasCode = $derived(html.includes("<pre"));
    const hasTags = $derived(data.post.tags.length > 0);
    const hasAdjacent = $derived(
        Boolean(data.adjacent.previous || data.adjacent.next),
    );
    const hasRelated = $derived(data.related.length > 0);
</script>

<div class="page">
    <section class="breadcrumb" aria-label="Breadcrumb">
        <a class="back-link" href="/blogs">← back to blogs</a>
        <p class="breadcrumb-path">
            <span class="path-qasim">qasim</span><span class="path-sk"
                >sk20</span
            >
            <span class="path-separator">/blogs/</span><span class="path-slug"
                >{data.post.slug}</span
            >
        </p>
        <span class="locale">{data.meta.locale}</span>
    </section>

    <header class="hero">
        <h1 class="title">{data.post.title}</h1>

        <div class="meta">
            <span>{data.post.publishedLabel}</span>
            <span aria-hidden="true">·</span>
            <span>{data.post.readingTime}</span>
        </div>

        {#if hasTags}
            <ul class="tag-list" aria-label="Post tags">
                {#each data.post.tags as tag}
                    <li
                        class="tag-item tag-{tag.color}"
                        data-badge
                        data-variant={tag.color}
                    >
                        <span>{tag.name}</span>
                    </li>
                {/each}
            </ul>
        {/if}
    </header>

    <main class="content">
        <article class="body" aria-label="Post body">
            <div class="post-body markdown" class:has-code={hasCode}>
                {@html html}
            </div>
        </article>

        {#if hasAdjacent}
            <nav class="adjacent" aria-label="Adjacent posts">
                {#if data.adjacent.previous}
                    <a
                        class="adjacent-link prev"
                        href={`/blogs/${data.adjacent.previous.slug}`}
                    >
                        <span class="adjacent-label">previous</span>
                        <span class="adjacent-title"
                            >{data.adjacent.previous.title}</span
                        >
                    </a>
                {:else}
                    <span class="adjacent-placeholder prev"
                        >start of archive</span
                    >
                {/if}

                {#if data.adjacent.next}
                    <a
                        class="adjacent-link next"
                        href={`/blogs/${data.adjacent.next.slug}`}
                    >
                        <span class="adjacent-label">next</span>
                        <span class="adjacent-title"
                            >{data.adjacent.next.title}</span
                        >
                    </a>
                {:else}
                    <span class="adjacent-placeholder next">end of archive</span
                    >
                {/if}
            </nav>
        {/if}

        {#if hasRelated}
            <section class="related" aria-label="Related posts">
                <h2>related posts</h2>
                <ul class="related-list">
                    {#each data.related as item (item.slug)}
                        <li>
                            <a href={`/blogs/${item.slug}`}>
                                <span class="related-title">{item.title}</span>
                                <span class="related-meta"
                                    >{item.publishedLabel} · {item.readingTime}</span
                                >
                            </a>
                        </li>
                    {/each}
                </ul>
            </section>
        {/if}
    </main>

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
        max-width: 920px;
        margin: 0 auto;
        padding: 4rem 1.5rem 3rem;
        display: flex;
        flex-direction: column;
        gap: 3rem;
    }

    .breadcrumb {
        display: flex;
        justify-content: space-between;
        align-items: center;
        color: var(--subtext1);
        text-transform: lowercase;
        letter-spacing: 0.15ch;
        font-size: 0.9rem;
        gap: 1rem;
        flex-wrap: wrap;
    }

    .back-link {
        color: var(--lavender);
        text-decoration: none;
        flex-shrink: 0;
    }

    .back-link:hover {
        text-decoration: underline;
    }

    .breadcrumb-path {
        margin: 0;
        font-size: 0.85rem;
        letter-spacing: 0.12ch;
        flex: 1;
        text-align: center;
    }

    .path-qasim {
        color: var(--teal);
    }

    .path-sk {
        color: var(--lavender);
    }

    .path-separator {
        color: var(--subtext1);
    }

    .path-slug {
        color: var(--text);
    }

    .locale {
        color: var(--subtext1);
        flex-shrink: 0;
    }

    .hero {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        text-align: center;
        margin-top: 2rem;
    }

    .title {
        margin: 0;
        font-size: clamp(2rem, 5vw, 3.2rem);
        font-weight: 600;
        letter-spacing: 0.01em;
        color: var(--text);
    }

    .meta {
        display: flex;
        gap: 1rem;
        justify-content: center;
        color: var(--subtext1);
        text-transform: lowercase;
        letter-spacing: 0.18ch;
        font-size: 0.95rem;
    }

    .tag-list {
        margin: 0;
        padding: 0;
        list-style: none;
        display: flex;
        gap: 0.75rem;
        justify-content: center;
        flex-wrap: wrap;
    }

    .tag-list li::before {
        content: none;
    }

    .tag-item {
        display: flex;
        padding: 0.4rem 1rem;
        letter-spacing: 0.12ch;
        text-transform: lowercase;
        font-size: 0.9rem;
    }

    .tag-rosewater {
        background: var(--rosewater);
        color: var(--base);
    }

    .tag-flamingo {
        background: var(--flamingo);
        color: var(--base);
    }

    .tag-pink {
        background: var(--pink);
        color: var(--base);
    }

    .tag-mauve {
        background: var(--mauve);
        color: var(--base);
    }

    .tag-red {
        background: var(--red);
        color: var(--base);
    }

    .tag-maroon {
        background: var(--maroon);
        color: var(--base);
    }

    .tag-peach {
        background: var(--peach);
        color: var(--base);
    }

    .tag-yellow {
        background: var(--yellow);
        color: var(--base);
    }

    .tag-green {
        background: var(--green);
        color: var(--base);
    }

    .tag-teal {
        background: var(--teal);
        color: var(--base);
    }

    .tag-sky {
        background: var(--sky);
        color: var(--base);
    }

    .tag-sapphire {
        background: var(--sapphire);
        color: var(--base);
    }

    /* markdown styles */
    :global(.markdown pre) {
        background: var(--surface0);
        border: 1px solid var(--surface1);
        padding: 1rem;
        overflow: auto;
    }
    :global(.markdown code) {
        font-family:
            ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas,
            "Liberation Mono", "Courier New", monospace;
        font-size: 0.95rem;
    }
    :global(.markdown pre code.hljs) {
        display: block;
        color: var(--text);
    }
    :global(.markdown h1),
    :global(.markdown h2),
    :global(.markdown h3) {
        margin: 1.2rem 0 0.6rem;
    }
    :global(.markdown p) {
        margin: 0.5rem 0;
    }
    :global(.markdown a) {
        color: var(--lavender);
    }

    .tag-blue {
        background: var(--blue);
        color: var(--base);
    }

    .tag-lavender {
        background: var(--lavender);
        color: var(--base);
    }

    .post-body {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    :global(.post-body p) {
        margin: 0;
        line-height: 1.75;
        font-family: inherit;
        font-size: 1.05rem;
        color: var(--text);
        letter-spacing: 0.01em;
    }

    .adjacent {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
        gap: 1.5rem;
        border: 1px solid var(--surface1);
        padding: 1.75rem;
    }

    .adjacent-link,
    .adjacent-placeholder {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
        text-decoration: none;
        color: var(--text);
    }

    .adjacent-link:hover .adjacent-title {
        text-decoration: underline;
    }

    .adjacent-label {
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.35ch;
        color: var(--subtext1);
    }

    .adjacent-title,
    .adjacent-placeholder {
        font-size: 1rem;
        color: var(--subtext0);
        letter-spacing: 0.06ch;
        text-transform: lowercase;
    }

    .adjacent-placeholder {
        opacity: 0.6;
    }

    .related {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        border: 1px solid var(--surface1);
        padding: 1.75rem;
    }

    .related h2 {
        margin: 0;
        font-size: 1rem;
        text-transform: uppercase;
        letter-spacing: 0.35ch;
        color: var(--subtext0);
    }

    .related-list {
        margin: 0;
        padding: 0;
        list-style: none;
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .related-list a {
        text-decoration: none;
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        color: var(--text);
    }

    .related-list a:hover .related-title {
        text-decoration: underline;
    }

    .related-title {
        font-size: 1.05rem;
    }

    .related-meta {
        font-size: 0.85rem;
        color: var(--subtext1);
        text-transform: lowercase;
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

        .breadcrumb {
            flex-direction: column;
            align-items: flex-start;
            gap: 0.75rem;
        }

        .breadcrumb-path {
            text-align: left;
        }

        .meta {
            flex-direction: column;
            gap: 0.5rem;
        }

        .tag-list {
            flex-wrap: wrap;
        }

        .body {
            padding: 1.75rem 1.5rem;
        }
    }
</style>
