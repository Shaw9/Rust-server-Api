# Repository Guidelines

## Project Structure & Module Organization
`src/lib.rs` contains the native addon entry points exposed through `napi-rs`; keep Windows-specific logic and FFI helpers close to the exported function they support. `__test__/` holds AVA integration tests for the JavaScript-facing API with files such as `index.spec.ts`, and `benchmark/` contains Tinybench performance checks. Top-level `package.json` drives Node-side tooling, while `Cargo.toml` and `build.rs` define the Rust crate. Treat `dist/`, `target/`, and generated `*.node` binaries as build output, not hand-edited source.

## Build, Test, and Development Commands
Run commands from the repository root.

- `bun install`: install JavaScript tooling and hooks.
- `bun run build`: compile the release addon into `dist/native`.
- `bun run build:debug`: build a debug binary for local troubleshooting.
- `bun run test`: run AVA integration tests through the Node loader.
- `cargo test`: run Rust unit tests, including helpers in `src/lib.rs`.
- `bun run bench`: compare native and JavaScript implementations with Tinybench.
- `bun run lint` and `bun run format`: run Oxc linting plus Prettier, `cargo fmt`, and Taplo formatting.

## Coding Style & Naming Conventions
Use 2-space indentation, LF line endings, UTF-8, and final newlines per `.editorconfig`. JavaScript and TypeScript follow Prettier settings: single quotes, no semicolons, trailing commas, and a 120-column wrap. Keep Rust code `cargo fmt` clean and prefer small, focused helpers for Win32 interop. Use `snake_case` for Rust items and descriptive lowercase filenames such as `index.spec.ts` and `bench.ts`.

## Testing Guidelines
Add JavaScript-facing regression tests under `__test__/` using AVA and name them `*.spec.ts`. Keep low-level Rust tests near the implementation with `#[cfg(test)]`. When changing exported functions or Windows notification behavior, run both `bun run test` and `cargo test`; if performance is relevant, also run `bun run bench`. There is no enforced coverage threshold, so prioritize tests that exercise real addon behavior.

## Commit & Pull Request Guidelines
Follow Conventional Commits, matching the current history: `feat(Windows): add notification icon decoding`, `refactor(api): simplify base64 parsing`. Keep subjects short and imperative. Pull requests should summarize API changes, note Windows-specific verification, link related issues, and avoid committing generated `dist/`, `target/`, or local binary artifacts.

## Platform Notes
This crate is Windows-only (`#![cfg(windows)]`). Validate Win32 behavior on a Windows machine before merging, and review new native dependencies carefully.
