import 'pixi.js/browser';
import 'pixi.js/app';
import 'pixi.js/events';
import 'pixi.js/dom';
import 'pixi.js/filters';
import 'pixi.js/prepare';
import 'pixi.js/graphics';
import 'pixi.js/sprite-tiling';
import 'pixi.js/text';
import 'pixi.js/text-bitmap';
import 'pixi.js/text-html';
import 'pixi.js/mesh';
import 'pixi.js/particle-container';
import 'pixi.js/accessibility';
import 'pixi.js/advanced-blend-modes';

import {
  Application,
  Assets,
  Graphics,
  Sprite,
  TextureStyle,
  WebGLRenderer,
  isWebGLSupported,
  isWebGPUSupported,
} from 'pixi.js';
import init, { Game, InputState } from '../pkg/koth_core.js';

window.addEventListener('error', (event) => {
  console.error('[boot] window error', event.error ?? event.message ?? event, event);
});
window.addEventListener('unhandledrejection', (event) => {
  console.error('[boot] unhandled rejection', event.reason ?? event, event);
});

globalThis.__PIXI_APP_INIT__ = (app, version) => {
  console.log('[boot] pixi app init hook', { version, app });
};
globalThis.__PIXI_RENDERER_INIT__ = (renderer, version) => {
  console.log('[boot] pixi renderer init hook', {
    version,
    type: renderer?.type,
    name: renderer?.name,
  });
};

const mountEl = document.getElementById('app');
const view = document.createElement('canvas');
if (mountEl) {
  mountEl.appendChild(view);
}
view.addEventListener('webglcontextlost', (event) => {
  event.preventDefault();
  console.error('[boot] view context lost', event);
});
view.addEventListener('webglcontextrestored', (event) => {
  console.warn('[boot] view context restored', event);
});
const app = new Application();
const bootStartedAt = performance.now();
console.log('[boot] starting app.init', {
  mode: import.meta.env.MODE,
  baseUrl: import.meta.env.BASE_URL,
});
console.log('[boot] isWebGLSupported', isWebGLSupported());
console.log('[boot] navigator.gpu', Boolean(navigator.gpu));
const webgpuSupportProbe = Promise.race([
  isWebGPUSupported(),
  new Promise((resolve) => setTimeout(() => resolve('timeout'), 1500)),
]);
console.log('[boot] isWebGPUSupported', await webgpuSupportProbe);
const preflightCanvas = document.createElement('canvas');
const preflight = {
  webgl2: Boolean(preflightCanvas.getContext('webgl2')),
  webgl: Boolean(preflightCanvas.getContext('webgl')),
};
console.log('[boot] webgl preflight', preflight);
const viewContext = view.getContext('webgl2', {
  alpha: true,
  antialias: true,
  stencil: true,
  powerPreference: 'high-performance',
});
console.log('[boot] view context', {
  webgl2: Boolean(viewContext),
});
setTimeout(() => {
  const resources = performance.getEntriesByType('resource');
  const interesting = resources
    .filter((entry) => /WebGLRenderer|WebGPURenderer|SharedSystems|index-/.test(entry.name))
    .map((entry) => ({
      name: entry.name,
      startTime: Math.round(entry.startTime),
      duration: Math.round(entry.duration),
      transferSize: entry.transferSize,
    }));
  console.log('[boot] resource snapshot', interesting);
}, 1000);
const appInitTimeout = setTimeout(() => {
  console.warn('[boot] app.init timeout (>5s)', {
    elapsedMs: Math.round(performance.now() - bootStartedAt),
  });
}, 5000);
console.log('[boot] WebGLRenderer init probe start');
console.log('[boot] renderer init systems', WebGLRenderer?.defaultSystemConfig?.systems?.length ?? 'unknown');
const probeRenderer = new WebGLRenderer();
console.log('[boot] runner init items', probeRenderer.runners.init.items.length);
for (const system of probeRenderer.runners.init.items) {
  if (typeof system?.init === 'function') {
    const originalInit = system.init.bind(system);
    system.init = async (options) => {
      const label = system.constructor?.name ?? 'unknown-system';
      const startedAt = performance.now();
      console.log('[boot] system init start', label);
      try {
        const result = await originalInit(options);
        console.log('[boot] system init done', label, {
          elapsedMs: Math.round(performance.now() - startedAt),
        });
        return result;
      } catch (error) {
        console.error('[boot] system init failed', label, error);
        throw error;
      }
    };
  }
}
const probeInit = Promise.race([
  probeRenderer.init({
    view,
    context: viewContext ?? undefined,
    preferWebGLVersion: 2,
    background: '#0b0b0b',
    resizeTo: window,
    preference: 'webgl',
    powerPreference: 'high-performance',
  }),
  new Promise((resolve) => setTimeout(() => resolve('timeout'), 3000)),
]);
const probeResult = await probeInit;
if (probeResult === 'timeout') {
  console.warn('[boot] WebGLRenderer init probe timeout (>3s)');
} else {
  console.log('[boot] WebGLRenderer init probe complete');
}
await app.init({
  view,
  context: viewContext ?? undefined,
  preferWebGLVersion: 2,
  background: '#0b0b0b',
  resizeTo: window,
  preference: 'webgl',
  powerPreference: 'high-performance',
});
clearTimeout(appInitTimeout);
console.log('[boot] app.init complete', {
  renderer: app.renderer?.constructor?.name ?? 'unknown',
});

TextureStyle.defaultOptions.scaleMode = 'nearest';

app.canvas.addEventListener('webglcontextlost', (event) => {
  event.preventDefault();
  console.error('WebGL context lost', event);
});
app.canvas.addEventListener('webglcontextrestored', () => {
  console.warn('WebGL context restored');
});

console.info('Pixi renderer:', app.renderer?.constructor?.name ?? 'unknown');

await Assets.init({
  basePath: import.meta.env.BASE_URL ?? '/',
});

const spritePaths = [
  'sprites/retro-triguy-stride1-sprite.png',
  'sprites/retro-triguy-strike-sprite.png',
  'sprites/retro-wedgeguy-stride-sprite.png',
  'sprites/retro-wedgeguy-strike-sprite.png',
  'sprites/retro-castle-sprite.png',
  'sprites/retro-cannon-sprite.png',
  'sprites/retro-crossbow-sprite.png',
  'sprites/retro-arrow-sprite.png',
  'sprites/retro-cannonball-sprite.png',
];

let textures = [];
try {
  await Assets.load(spritePaths);
  textures = spritePaths.map((path) => Assets.get(path));
} catch (error) {
  console.error('Failed to load sprite assets', error);
}

await init();
const game = new Game();
const input = new InputState();
const scoreEl = document.getElementById('score');
const wallEl = document.getElementById('wall');
const statusEl = document.getElementById('status');
const gameOverEl = document.getElementById('game-over');
const restartButton = document.getElementById('restart');
let lastScore = -1;
let lastWall = -1;
let lastGameOver = null;

if (restartButton) {
  restartButton.addEventListener('click', () => {
    game.restart();
    lastScore = -1;
    lastWall = -1;
    lastGameOver = null;
  });
}

const spritesByEntity = [];
const debugLayer = new Graphics();
app.stage.addChild(debugLayer);

let lastViewport = { w: 0, h: 0 };
const readViewport = () => ({
  w: app.renderer?.width ?? app.canvas.width,
  h: app.renderer?.height ?? app.canvas.height,
});
window.addEventListener('resize', () => {
  lastViewport = { w: 0, h: 0 };
});

const pointer = {
  x: 0,
  y: 0,
  down: false,
};

app.canvas.addEventListener('pointermove', (e) => {
  const rect = app.canvas.getBoundingClientRect();
  pointer.x = e.clientX - rect.left;
  pointer.y = e.clientY - rect.top;
});
app.canvas.addEventListener('pointerdown', () => {
  pointer.down = true;
});
app.canvas.addEventListener('pointerup', () => {
  pointer.down = false;
});
app.canvas.addEventListener('pointerleave', () => {
  pointer.down = false;
});

app.ticker.add((ticker) => {
  const viewport = readViewport();
  if (viewport.w > 0 && viewport.h > 0) {
    if (viewport.w !== lastViewport.w || viewport.h !== lastViewport.h) {
      game.set_viewport(viewport.w, viewport.h);
      lastViewport = viewport;
    }
  }
  input.pointer_x = pointer.x;
  input.pointer_y = pointer.y;
  input.pointer_down = pointer.down;

  const dt = ticker.deltaMS / 1000;
  game.tick(dt, input);

  const score = game.score();
  if (score !== lastScore && scoreEl) {
    scoreEl.textContent = String(score);
    lastScore = score;
  }
  const wallIntegrity = game.wall_integrity();
  const wallPercent = Math.max(0, Math.min(100, Math.round(wallIntegrity * 100)));
  if (wallPercent !== lastWall && wallEl) {
    wallEl.textContent = `${wallPercent}%`;
    lastWall = wallPercent;
  }
  const gameOver = game.game_over();
  const gameOverChanged = gameOver !== lastGameOver;
  if (gameOverChanged && statusEl) {
    statusEl.textContent = gameOver ? 'DEFEAT' : '';
  }
  if (gameOverEl && gameOverChanged) {
    gameOverEl.classList.toggle('is-visible', gameOver);
    gameOverEl.setAttribute('aria-hidden', gameOver ? 'false' : 'true');
  }
  if (gameOverChanged) {
    lastGameOver = gameOver;
  }

  const renderList = game.render_list();
  const debugList = game.debug_list();
  const base = Math.min(app.canvas.width, app.canvas.height);
  const castleHeight = base * 0.25;
  const weaponHeight = castleHeight * 0.2;
  const characterHeight = castleHeight * 0.35;
  const projectileHeight = weaponHeight * 0.35;

  const desiredHeightForSprite = (spriteId) => {
    switch (spriteId) {
      case 4:
        return castleHeight;
      case 5:
      case 6:
        return weaponHeight;
      case 7:
      case 8:
        return projectileHeight;
      default:
        return characterHeight;
    }
  };

  for (let i = 0; i + 3 < renderList.length; i += 4) {
    const x = renderList[i];
    const y = renderList[i + 1];
    const rotation = renderList[i + 2];
    const spriteId = renderList[i + 3];
    const entityIndex = i / 4;

    let sprite = spritesByEntity[entityIndex];
    const tex = textures[spriteId] ?? textures[0];
    if (!sprite) {
      sprite = new Sprite(tex);
      sprite.anchor.set(0.5, 0.5);
      spritesByEntity[entityIndex] = sprite;
      app.stage.addChild(sprite);
    }

    if (sprite.texture !== tex) {
      sprite.texture = tex;
    }
    const targetHeight = desiredHeightForSprite(spriteId);
    const scale = targetHeight / sprite.texture.height;
    const flipX = spriteId === 7 ? -1 : 1;
    sprite.scale.set(scale * flipX, scale);

    sprite.x = x;
    sprite.y = y;
    sprite.rotation = rotation;
  }

  debugLayer.clear();
  debugLayer.strokeStyle = { width: 1, color: 0x00ff66, alpha: 0.8 };
  for (let i = 0; i + 4 < debugList.length; i += 5) {
    const kind = debugList[i];
    const x = debugList[i + 1];
    const y = debugList[i + 2];
    const a = debugList[i + 3];
    const b = debugList[i + 4];

    if (kind === 0) {
      debugLayer.circle(x, y, a);
      debugLayer.stroke();
    } else if (kind === 1) {
      debugLayer.rect(x - a, y - b, a * 2, b * 2);
      debugLayer.stroke();
    }
  }
});
