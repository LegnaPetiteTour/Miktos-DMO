// @sveltejs/vite-plugin-svelte applies vitePreprocess() automatically via vite.config.ts.
// An explicit import here breaks the Svelte language server in monorepo setups;
// Svelte 5 handles TypeScript natively without it.
export default {};
