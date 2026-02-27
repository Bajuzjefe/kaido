# Kaido Web Wizard

Browser-based smart contract wizard for Kaido. Built with React 19, Vite, Tailwind CSS 4, and WebAssembly.

## Development

```bash
# Dev mode (fallback code generation, no WASM needed)
npm run dev

# Build WASM module (requires Rust + wasm32 target)
npm run build:wasm

# Verify WASM artifact has no local path leakage
npm run check:public-artifacts

# Full production build (WASM + Vite)
npm run build:full

# Preview production build
npm run preview
```

## Architecture

- **WASM module** (`src/wasm/`) — kaido-core compiled to WebAssembly via `wasm-bindgen`. Generates Aiken code in-browser with zero server calls.
- **Fallback mode** — when WASM isn't built (dev), generates sample Aiken output for UI development.
- **Shiki** — syntax highlighting with github-dark-default theme. Aiken highlighted via Rust grammar.

## Deployment

Deployed to Vercel as a static site. WASM is pre-built in CI and included in the output.

Important: Vercel build uses the committed `src/wasm/*` bundle. If `crates/kaido-core` template
metadata changes (for example `supports_sdk`), regenerate and commit WASM artifacts first:

```bash
npm run build:wasm
npm run check:public-artifacts
git add src/wasm
```

```bash
# Local deploy (requires vercel CLI auth)
npx vercel build --prod && npx vercel deploy --prod --prebuilt
```
