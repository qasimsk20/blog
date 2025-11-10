<div class="bento-item">
    <slot />
</div>

<style>
    .bento-item {
        border: 1px solid var(--border-color);
        background-color: var(--mantle);
        padding: 1.5rem;
    }

    .bento-item h3 {
        margin-top: 0;
        color: var(--accent-color);
    }
</style>
