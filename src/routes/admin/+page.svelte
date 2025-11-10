<script lang="ts">
    const hints = [
        "this area is monitored",
        "pure terminal aesthetics only",
        "authorized operators know the true entrypoint",
        "access requires proper credentials",
    ];

    const randomIndex = Math.floor(Math.random() * hints.length);
    const hint = hints[randomIndex];

    let attempt = $state("");
    let status = $state<"idle" | "processing" | "denied">("idle");

    function handleSubmit() {
        if (!attempt.trim()) {
            status = "idle";
            return;
        }

        status = "processing";

        setTimeout(() => {
            status = "denied";
        }, 420);
    }
</script>

<div class="page">
    <section class="panel">
        <header class="header">
            <h1 class="title">admin / access denied</h1>
            <p class="subtitle">unauthorized entry detected</p>
        </header>

        <form
            class="auth-form"
            role="presentation"
            on:submit|preventDefault={handleSubmit}
            aria-describedby="auth-hint"
        >
            <label class="form-label" for="admin-attempt"
                >operator passphrase</label
            >
            <div class="field-group">
                <input
                    id="admin-attempt"
                    name="admin-attempt"
                    type="password"
                    placeholder="••••••••"
                    bind:value={attempt}
                    aria-describedby="auth-status"
                    autocomplete="off"
                />
                <button type="submit">verify</button>
            </div>
            <p id="auth-status" class="status {status}">
                {#if status === "idle"}
                    awaiting valid operator credentials
                {:else if status === "processing"}
                    verifying...
                {:else}
                    access denied. invalid credentials.
                {/if}
            </p>
        </form>

        <section class="message">
            <p class="hint-label">terminal hint</p>
            <p id="auth-hint" class="hint-value">{hint}</p>
        </section>

        <section class="notice">
            <p>
                This interface is for demonstration purposes only. Unauthorized
                access attempts are logged and monitored. Authorized operators
                have secure access through alternative channels.
            </p>
        </section>
    </section>
</div>

<style>
    .page {
        min-height: 100vh;
        display: flex;
        justify-content: center;
        align-items: center;
        padding: 4rem 1.5rem;
        background: var(--base);
        color: var(--text);
    }

    .panel {
        width: min(520px, 100%);
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
        padding: 2.75rem;
        border: 1px solid var(--surface1);
        background: color-mix(in srgb, var(--surface0) 40%, transparent);
        box-shadow: 0 0 0 1px
            color-mix(in srgb, var(--surface1) 35%, transparent);
    }

    .header {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        text-transform: lowercase;
        letter-spacing: 0.14ch;
    }

    .title {
        margin: 0;
        font-size: clamp(1.6rem, 4vw, 2rem);
        font-weight: 600;
        color: var(--lavender);
    }

    .subtitle {
        margin: 0;
        color: var(--subtext0);
        font-size: 0.95rem;
    }

    .auth-form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .form-label {
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.3ch;
        color: var(--subtext1);
    }

    .field-group {
        display: flex;
        gap: 0.75rem;
        align-items: center;
    }

    .field-group input {
        flex: 1;
        background: transparent;
        border: none;
        border-bottom: 2px solid var(--surface1);
        padding: 0.65rem 0;
        color: var(--text);
        font-family: inherit;
        font-size: 1rem;
        letter-spacing: 0.1ch;
    }

    .field-group input:focus-visible {
        outline: none;
        border-bottom-color: var(--lavender);
    }

    .field-group button {
        padding: 0.55rem 1.25rem;
        border: 1px solid var(--surface1);
        background: transparent;
        color: var(--peach);
        font-family: inherit;
        font-size: 0.95rem;
        text-transform: lowercase;
        letter-spacing: 0.14ch;
        cursor: pointer;
        transition:
            border-color 0.2s ease,
            color 0.2s ease;
    }

    .field-group button:hover {
        border-color: var(--peach);
        color: var(--peach);
    }

    .status {
        margin: 0;
        font-size: 0.9rem;
        color: var(--subtext1);
        text-transform: lowercase;
        letter-spacing: 0.1ch;
        min-height: 1.4rem;
    }

    .status.processing {
        color: var(--yellow);
    }

    .status.denied {
        color: var(--peach);
    }

    .message {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
        border: 1px solid var(--surface1);
        padding: 1.25rem;
        background: color-mix(in srgb, var(--surface0) 25%, transparent);
    }

    .hint-label {
        margin: 0;
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.32ch;
        color: var(--subtext1);
    }

    .hint-value {
        margin: 0;
        color: var(--text);
        letter-spacing: 0.1ch;
        text-transform: lowercase;
    }

    .notice {
        border-top: 1px solid var(--surface1);
        padding-top: 1.25rem;
        color: var(--subtext0);
        font-size: 0.9rem;
        letter-spacing: 0.08ch;
        text-transform: lowercase;
    }

    @media (max-width: 520px) {
        .panel {
            padding: 2.25rem 2rem;
        }

        .field-group {
            flex-direction: column;
            align-items: stretch;
        }

        .field-group button {
            width: 100%;
        }

        .message,
        .notice {
            font-size: 0.88rem;
        }
    }
</style>
