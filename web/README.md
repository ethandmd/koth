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

## Render list format
The render list is a flat `Float32Array` in triples:

```
[x, y, sprite_id]
```

`main.js` interprets each triple as a single entity. `sprite_id` indexes into the `spritePaths` list.

## Current demo behavior
- A castle sprite is centered in the viewport.
- A triguy sprite advances from the left toward center, alternating stride/strike.
- A wedgeguy sprite advances from the right toward center, alternating stride/strike.

## Key files
- `web/src/main.js` — Pixi setup, input capture, render loop
- `web/public/sprites/` — sprite assets
- `web/pkg/` — WASM build output (generated)
