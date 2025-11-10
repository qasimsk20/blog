<style>
    .bento-box {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 1rem;
        padding: 1rem 0;
    }
</style>

<div class="bento-box">
    <slot />
</div>
