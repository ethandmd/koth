# web/

## What this does
This directory hosts the browser client (Vite + PixiJS). It loads the Rust WASM core, forwards input, and renders sprites based on the render list returned by Rust.

## Build / run (pnpm)
From this directory:

```sh
pnpm install
pnpm run build:wasm
pnpm run dev
```

- `build:wasm` compiles `../rs` into `web/pkg/` via `wasm-pack`.
- `dev` starts Vite at http://localhost:5173

## Asset layout
Static sprites live in `web/public/sprites/` and are referenced by absolute paths like:

```
/sprites/retro-triguy-stride1-sprite.png
```

## Runtime flow
1) `main.js` loads the WASM bindings from `web/pkg/`.
2) It initializes Pixi and preloads sprite textures.
3) Each frame:
   - Pointer input is captured from DOM events.
   - Input is passed to Rust via `game.tick(dt, input)`.
   - Rust returns a packed `Float32Array` render list.
   - JS maps each entity to a Pixi `Sprite` and updates position/texture.
   - Rust returns a packed `Float32Array` debug list for colliders.
   - HUD reads `score`, `wall_integrity`, and `game_over`.

## Render list format
The render list is a flat `Float32Array` in quads:

```
[x, y, rotation, sprite_id]
```

`main.js` interprets each quad as a single entity. `sprite_id` indexes into the `spritePaths` list.

## Debug list format
The debug list is a flat `Float32Array` in quintuples:

```
[kind, x, y, a, b]
```

`kind` values:
- `0`: circle (`a = radius`)
- `1`: AABB (`a = half_width`, `b = half_height`)

## Current demo behavior
- A castle sprite is centered in the viewport.
- A triguy sprite advances from the left toward center, alternating stride/strike.
- A wedgeguy sprite advances from the right toward center, alternating stride/strike.
- Enemies stop at the wall and continuously damage wall integrity.
- A game over overlay appears at 0% wall integrity with a restart button.
- Pointer-down fires the left crossbow or right cannon toward the cursor.
- Projectiles arc under gravity and score increments on hits.

## Key files
- `web/src/main.js` — Pixi setup, input capture, render loop
- `web/public/sprites/` — sprite assets
- `web/pkg/` — WASM build output (generated)

## PixiJS init imports (important for production builds)
Condition:
- `pnpm run build && pnpm run preview` (or production bundle) hangs during `app.init()` and never renders.

Cause:
- Pixi’s renderer systems can be tree-shaken away in production, and Pixi’s internal WebGL context creation can hang.

Correction:
- Keep explicit side-effect imports in `web/src/main.js` (e.g. `pixi.js/app`, `pixi.js/events`, `pixi.js/graphics`, etc.) so renderer systems are registered.
- Create a canvas and WebGL2 context up front, then pass both into `app.init({ view, context, preferWebGLVersion: 2, ... })`.
