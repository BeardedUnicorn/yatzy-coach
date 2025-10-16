# Repository Guidelines

## Project Structure & Module Organization
- `src/` holds the React frontend; `main.tsx` bootstraps the root component defined in `App.tsx`.
- `public/` serves static assets bundled by Vite, while `src/assets/` stores build-time SVGs.
- `src-tauri/` contains the Rust shell: `src-tauri/src/main.rs` opens the window and `lib.rs` exposes commands to the UI.
- Tooling and packaging configs live at the repo root (`vite.config.ts`, `tsconfig*.json`, `src-tauri/tauri.conf.json`).

## Build, Test, and Development Commands
- `pnpm dev` starts the Vite dev server with fast refresh for the browser UI.
- `pnpm tauri dev` launches the desktop shell and keeps both layers hot-reloading.
- `pnpm build` runs TypeScript checks and creates a production bundle in `dist/`; use `pnpm preview` to verify that bundle.
- Inside `src-tauri/`, reach for `cargo fmt`, `cargo clippy`, and `cargo test` when iterating on Rust code.

## Coding Style & Naming Conventions
- Stick to TypeScript strict mode; type props explicitly and return `JSX.Element` from components.
- Use 2-space indentation, group imports by origin, and favor PascalCase for components with camelCase hooks and helpers.
- Keep React effects isolated in `useEffect`; format Rust files with `cargo fmt` to stay idiomatic.

## Testing Guidelines
- No automated harness ships yet; run `pnpm tauri dev` and walk through scoring flows before merging.
- When introducing tests, add Vitest + React Testing Library cases under `src/__tests__/` and Rust units with `cargo test` in `src-tauri/src`.
- Target smoke coverage for scoring logic and backend commands; note manual checks in each PR until suites exist.

## Commit & Pull Request Guidelines
- Use short, imperative commit subjects (e.g., `Add bonus calculator`) and capture cross-layer details in the body.
- Reference related tickets in the footer (`Refs #42`) and keep each commit focused on one vertical change.
- Pull requests must list manual test steps (browser and desktop) and attach screenshots for UI updates.

## Tauri & Configuration Tips
- Adjust window options or permissions in `src-tauri/tauri.conf.json`, keeping secrets in environment variables instead.
- Declare new native capabilities under `src-tauri/capabilities/` so desktop builds stay aligned across platforms.
