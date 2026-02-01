# King of the Hill (Tower Defense)

Client-side 2D arcade-style tower defense game. Rust owns the simulation; a minimal JS host renders via PixiJS and forwards inputs to WASM.

## Structure

- `rs/` — Rust core: game loop, simulation, physics/collisions, pathing, and rules. Exposes a WASM API.
- `web/` — Browser host: Vite + PixiJS, input capture, and sprite updates from the Rust render list.
- `utils/` — Project utilities (asset tooling and helpers).
- `web/public/sprites/` — Sprite assets used by the game (PNG with transparency).
- `retro-*.png` — Source art assets (raw inputs for sprite processing).

## Build

### Prereqs

- Rust toolchain (`rustup`, `cargo`)
- `wasm-pack` (required for generating the WASM/JS glue)
- `pnpm` (for the web host)

Install `wasm-pack` system-wide (recommended):

```bash
cargo install wasm-pack
```

### Rust core

```bash
cargo build
```

Run from `rs/`.

### Web host (WASM + bundle)

```bash
pnpm install
pnpm run build
```

Run from `web/`. This runs `wasm-pack build ../rs --target web --out-dir ../web/pkg` and then the Vite build.

### Web dev server

```bash
pnpm run dev
```

Run from `web/` to start Vite at `http://localhost:5173`.

## Quick start (recommended)

```bash
cd web
pnpm install
pnpm run build:wasm
pnpm run dev
```

This compiles the Rust core to WASM (`web/pkg/`) and starts the dev server.

## Notes

- The Rust build used by the browser is produced by `wasm-pack` (via `pnpm run build:wasm`).
- The Rust core returns a packed render list; JS reads it as a `Float32Array` of `[x, y, sprite_id]` triples.
- Sprite processing helper: `utils/sprite_crop.py` (crop, remove background, pad to uniform canvas).

## Physics + gameplay model plan (hecs)

### Goals
- Deterministic, lightweight gameplay simulation suitable for tower defense.
- Simple collision/range checks (no heavy rigid-body engine).
- Clear separation between simulation data and render output.
- Easy to add debug overlays for colliders, ranges, and paths.

### Implementation strategy
- **ECS:** Use `hecs` as a minimal ECS for entities and components.
- **Core components:**
  - `Transform { pos: Vec2 }`
  - `Velocity { v: Vec2 }`
  - `Collider { shape: Circle | Aabb, layer, mask }`
  - `Health { hp }`
  - `Team { id }`
  - `Renderable { sprite_id }`
- **Simulation loop (per tick):**
  - Integrate positions from velocity.
  - Resolve simple collisions (projectile vs enemy, enemy vs base).
  - Apply tower targeting (range checks) and spawn projectiles.
  - Update lifetimes / despawn entities.
- **Spatial queries:**
  - Start with naive O(n^2) checks (small entity counts).
  - Upgrade to uniform grid or spatial hash if needed.
- **Render list:**
  - Build from `Renderable + Transform` each frame.
  - Keep it flat and packed for WASM → JS transfer.
- **Debug view (later):**
  - Emit a separate debug list for colliders and ranges.
  - JS renders debug primitives (circles/rects) via Pixi `Graphics`.
