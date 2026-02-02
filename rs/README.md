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
  - `ground_y()` → `f32` (shared ground line in viewport space)
  - `set_castle_sprite_bounds(sprite_w, sprite_h, min_x, min_y, max_x, max_y)`
  - `tick(dt, input)`
  - `render_list()` → `Float32Array` in JS
  - `render_list_view()` → `Float32Array` view in JS (no copy)
  - `debug_list()` → `Float32Array` in JS
  - `debug_list_view()` → `Float32Array` view in JS (no copy)
  - `sprite_paths()` → `String[]` in JS
  - `explosion_sprite_base()` → `u16` (first explosion sprite id)
  - `explosion_scales()` → `Float32Array` in JS
  - `sprite_heights()` → `Float32Array` in JS
  - `score()` → `u32`
  - `wall_integrity()` → `f32` (0.0 - 1.0)
  - `game_over()` → `bool`
  - `restart()`

## Render list format
Packed as a flat float array, grouped as:

```
[x, y, rotation, sprite_id, target_height]
```

Each frame, JS reads these quintuples and updates sprites.

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
- Triguy and wedgeguy approach the castle and stop at the castle wall.
- Each attacker at the wall damages wall integrity until it reaches 0%.
- Pointer input fires crossbow bolts (left side) or cannonballs (right side).
- Projectiles use simple gravity and circle hit tests; hits increment `score`.
- Cannonballs hitting the ground line (including the castle sink offset) trigger a brief expanding/shrinking explosion (50ms steps).

## Next steps (planned)
- Expand the render list schema (rotation, frame index, tint, etc.).
- Move to a zero-allocation render list (WASM memory pointer + length).
- Add deterministic simulation helpers for replay/debug.
