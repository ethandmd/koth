# rs/

## What this does
This directory holds the Rust gameplay core compiled to WebAssembly. It is the source of truth for game state, simulation, and physics. JavaScript only renders and forwards input.

## Structure
- `Cargo.toml`
  - Declares a `cdylib` crate so Rust can compile to WASM.
  - Dependencies:
    - `wasm-bindgen` for JS interop
    - `glam` for game math (future use)
- `src/lib.rs`
  - WASM entry point and exported API (`Game`, `InputState`).

## Build / compile
From the `web/` directory:

```sh
pnpm run build:wasm
```

This runs:

```sh
wasm-pack build ../rs --target web --out-dir ../web/pkg
```

Output:
- `web/pkg/koth_core_bg.wasm` (the compiled WASM module)
- `web/pkg/koth_core.js` (JS glue + bindings)

## Current API
- `InputState`
  - `pointer_x`, `pointer_y`, `pointer_down`
- `Game`
  - `new()`
  - `tick(dt, input)`
  - `render_list()` → `Float32Array` in JS

## Render list format
Packed as a flat float array, grouped as:

```
[x, y, sprite_id]
```

Each frame, JS reads these triples and updates sprites.

## Current logic
- `Game` stores `time` and advances it in `tick`.
- `render_list()` returns one placeholder entity that moves horizontally using `sin(time)`.

## Next steps (planned)
- Expand the render list schema (rotation, frame index, tint, etc.).
- Move to a zero-allocation render list (WASM memory pointer + length).
- Add deterministic simulation helpers for replay/debug.
