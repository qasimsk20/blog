import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),

  kit: {
    // adapter-static for Cloudflare Pages deployment
    adapter: adapter({
      // fallback for client-side routing
      fallback: "index.html",
    }),

    // Clean configuration for Cloudflare Pages
    // No base path needed - use custom domains
    paths: {
      base: process.env.BASE_PATH || "",
    },
  },
};

export default config;
