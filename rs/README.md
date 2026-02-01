# rs/

## What this does
This directory holds the Rust gameplay core compiled to WebAssembly. It is the source of truth for game state, simulation, and physics. JavaScript only renders and forwards input.

## Structure
- `Cargo.toml`
  - Declares a `cdylib` crate so Rust can compile to WASM.
  - Dependencies:
    - `wasm-bindgen` for JS interop
    - `glam` for game math
    - `hecs` for the ECS world
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
  - `set_viewport(width, height)`
  - `tick(dt, input)`
  - `render_list()` → `Float32Array` in JS
  - `debug_list()` → `Float32Array` in JS
  - `score()` → `u32`

## Render list format
Packed as a flat float array, grouped as:

```
[x, y, rotation, sprite_id]
```

Each frame, JS reads these quads and updates sprites.

## Debug list format
Packed as a flat float array, grouped as:

```
[kind, x, y, a, b]
```

`kind` values:
- `0`: circle (`a = radius`)
- `1`: AABB (`a = half_width`, `b = half_height`)

## Current logic
- `Game` stores `time` and advances it in `tick`.
- Castle, cannon, and crossbow are anchored to the viewport.
- Triguy and wedgeguy approach the castle and then fall offscreen.
- Pointer input fires crossbow bolts (left side) or cannonballs (right side).
- Projectiles use simple gravity and circle hit tests; hits increment `score`.

## Next steps (planned)
- Expand the render list schema (rotation, frame index, tint, etc.).
- Move to a zero-allocation render list (WASM memory pointer + length).
- Add deterministic simulation helpers for replay/debug.
