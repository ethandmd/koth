# AGENTS.md

## Project goal
Build a 2D arcade-style tower defense game that runs entirely client-side in the browser.

## Core framework decisions
- **Runtime target:** Browser only (no server required for gameplay).
- **Language:** Rust for gameplay/physics logic; minimal JavaScript for host/render.
- **WASM toolchain:** `wasm-bindgen` + `wasm-pack` for Rust ⇄ JS interop.
- **Renderer:** PixiJS (WebGL with Canvas2D fallback).
- **Build tooling:** Vite for the web host/bundling.

## Architecture (Pattern A)
- Rust owns the game loop, state, physics/collisions, pathing, and rules.
- JS captures DOM input events and forwards a compact input state to Rust each frame.
- Rust outputs a compact render list; JS updates Pixi sprites from that list.

## Constraints & principles
- Keep JS ↔ WASM calls minimal (one input update + one render list per frame).
- Prefer pure-Rust dependencies (avoid C/C++ wrappers).
- Prioritize predictable performance on desktop and mobile browsers.
- When adding a new feature, update this document with any new development how-tos, and update `rs/README.md` or `web/README.md` to reflect how the new code works.

## Best practices from recent RCAs
- PixiJS v8: load images via `Assets.load(...)` (not `Texture.fromURL`, which no longer exists).
- When updating PixiJS, check the v8 API docs/changelog for breaking changes in asset loading.
- Keep `web/pkg` generated via `wasm-pack` and ensure dev docs call out this step.

## Nice-to-have
- PWA-friendly structure for mobile installation.
- Deterministic simulation for replays and debugging.
