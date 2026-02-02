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
  Container,
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

await init();
const game = new Game();
const spritePaths = Array.from(game.sprite_paths());

let textures = [];
let warnedSpriteId = false;
try {
  await Assets.load(spritePaths);
  textures = spritePaths.map((path) => Assets.get(path));
} catch (error) {
  console.error('Failed to load sprite assets', error);
}
const input = new InputState();
const scoreEl = document.getElementById('score');
const wallEl = document.getElementById('wall');
const statusEl = document.getElementById('status');
const gameOverEl = document.getElementById('game-over');
const restartButton = document.getElementById('restart');
let lastScore = -1;
let lastWall = -1;
let lastGameOver = null;

const resolveTextureImage = (texture) => {
  const source =
    texture?.source?.resource ??
    texture?.baseTexture?.resource ??
    texture?.resource ??
    null;
  return source?.source ?? source ?? null;
};

const computeOpaqueBounds = (image) => {
  if (!image || !image.width || !image.height) {
    return null;
  }
  const canvas = document.createElement('canvas');
  canvas.width = image.width;
  canvas.height = image.height;
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  if (!ctx) {
    return null;
  }
  ctx.drawImage(image, 0, 0);
  const { data, width, height } = ctx.getImageData(0, 0, canvas.width, canvas.height);
  let minX = width;
  let minY = height;
  let maxX = -1;
  let maxY = -1;
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const alpha = data[(y * width + x) * 4 + 3];
      if (alpha > 0) {
        if (x < minX) minX = x;
        if (y < minY) minY = y;
        if (x > maxX) maxX = x;
        if (y > maxY) maxY = y;
      }
    }
  }
  if (maxX < minX || maxY < minY) {
    return null;
  }
  return {
    spriteW: width,
    spriteH: height,
    minX,
    minY,
    maxX,
    maxY,
  };
};

const castleTexture = textures[4];
if (castleTexture) {
  const castleImage = resolveTextureImage(castleTexture);
  const bounds = computeOpaqueBounds(castleImage);
  if (bounds) {
    game.set_castle_sprite_bounds(
      bounds.spriteW,
      bounds.spriteH,
      bounds.minX,
      bounds.minY,
      bounds.maxX,
      bounds.maxY,
    );
  } else {
    console.warn('[boot] castle bounds unavailable, using default collider');
  }
}

if (restartButton) {
  restartButton.addEventListener('click', () => {
    game.restart();
    lastScore = -1;
    lastWall = -1;
    lastGameOver = null;
  });
}

const spritesByEntity = [];
const backgroundLayer = new Container();
const skyLayer = new Graphics();
const cloudLayer = new Container();
const groundLayer = new Graphics();
const treeLayer = new Graphics();
backgroundLayer.addChild(skyLayer);
backgroundLayer.addChild(cloudLayer);
backgroundLayer.addChild(groundLayer);
backgroundLayer.addChild(treeLayer);
app.stage.addChild(backgroundLayer);
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

const clamp = (value, min, max) => Math.min(max, Math.max(min, value));
const makeRng = (seed) => () => {
  seed = (seed * 1664525 + 1013904223) >>> 0;
  return seed / 4294967296;
};
const clouds = [];
const rebuildBackground = (viewport) => {
  const { w, h } = viewport;
  if (w <= 0 || h <= 0) {
    return;
  }
  const groundY = game.ground_y();
  if (!(groundY > 0)) {
    return;
  }
  const seed = (Math.round(w) * 73856093) ^ (Math.round(h) * 19349663);
  const rng = makeRng(seed >>> 0);

  skyLayer.clear();
  cloudLayer.removeChildren();
  groundLayer.clear();
  treeLayer.clear();
  clouds.length = 0;

  const skyMid = groundY * 0.6;
  skyLayer.rect(0, 0, w, skyMid).fill(0x69a9ff);
  skyLayer.rect(0, skyMid, w, groundY - skyMid).fill(0x8bc6ff);
  const sunRadius = Math.min(w, h) * 0.08;
  const sunX = w * 0.85;
  const sunY = h * 0.16;
  skyLayer.circle(sunX, sunY, sunRadius).fill(0xffe19a);
  skyLayer.circle(sunX, sunY, sunRadius * 1.35).fill({ color: 0xfff0c2, alpha: 0.35 });

  const cloudCount = clamp(Math.round(w / 180), 4, 8);
  for (let i = 0; i < cloudCount; i += 1) {
    const cloud = new Graphics();
    const size = clamp((0.7 + rng() * 0.9) * (w / 28), 24, 60);
    const puff = size * 0.5;
    cloud
      .circle(0, 0, puff)
      .circle(puff, -puff * 0.3, puff * 1.1)
      .circle(puff * 2, 0, puff * 0.9)
      .fill({ color: 0xffffff, alpha: 0.75 });
    cloud.x = rng() * w;
    cloud.y = clamp(rng() * groundY * 0.5 + h * 0.05, h * 0.05, groundY * 0.55);
    cloudLayer.addChild(cloud);
    clouds.push({
      sprite: cloud,
      speed: 6 + rng() * 10,
      width: puff * 2.8,
    });
  }

  const dirtColor = 0x7a4f2f;
  const grassColor = 0x3f8b3c;
  groundLayer.rect(0, groundY, w, h - groundY).fill(dirtColor);
  const grassDepth = clamp(h * 0.06, 18, 44);
  groundLayer.rect(0, groundY, w, grassDepth).fill(grassColor);
  groundLayer.rect(0, groundY, w, 3).fill(0x2d6b2d);

  const tuftCount = clamp(Math.round(w / 36), 14, 40);
  for (let i = 0; i < tuftCount; i += 1) {
    const x = rng() * w;
    const r = 3 + rng() * 4;
    groundLayer.circle(x, groundY + r * 0.6, r).fill(0x4d9a45);
  }

  const treeCount = clamp(Math.round(w / 220), 3, 7);
  for (let i = 0; i < treeCount; i += 1) {
    const x = (i + 0.3 + rng() * 0.4) * (w / treeCount);
    const trunkH = clamp(h * (0.12 + rng() * 0.12), 40, 120);
    const trunkW = clamp(trunkH * 0.18, 10, 26);
    const canopyR = trunkW * (2.3 + rng() * 0.9);
    treeLayer.rect(x - trunkW * 0.5, groundY - trunkH, trunkW, trunkH).fill(0x6b4a2d);
    const canopyX = x + (rng() - 0.5) * trunkW * 0.8;
    const canopyY = groundY - trunkH - canopyR * 0.45;
    treeLayer.circle(canopyX, canopyY, canopyR).fill(0x2f6f34);
    treeLayer.circle(canopyX + canopyR * 0.7, canopyY + canopyR * 0.15, canopyR * 0.75).fill(0x357b39);
    treeLayer.circle(canopyX - canopyR * 0.6, canopyY + canopyR * 0.2, canopyR * 0.65).fill(0x2b6230);
  }
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
      rebuildBackground(viewport);
      lastViewport = viewport;
    }
  }
  input.pointer_x = pointer.x;
  input.pointer_y = pointer.y;
  input.pointer_down = pointer.down;

  const dt = ticker.deltaMS / 1000;
  if (clouds.length > 0) {
    for (const cloud of clouds) {
      cloud.sprite.x += cloud.speed * dt;
      if (cloud.sprite.x - cloud.width > viewport.w + 40) {
        cloud.sprite.x = -cloud.width - 40;
      }
    }
  }
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

  const renderList = game.render_list_view();
  const debugList = game.debug_list_view();
  const stride = 6;
  for (let i = 0; i + (stride - 1) < renderList.length; i += stride) {
    const x = renderList[i];
    const y = renderList[i + 1];
    const rotation = renderList[i + 2];
    const spriteId = renderList[i + 3];
    let textureIndex = spriteId;
    if (textureIndex < 0 || textureIndex >= textures.length) {
      if (!warnedSpriteId) {
        console.warn('[render] sprite_id out of range', {
          spriteId,
          max: textures.length - 1,
        });
        warnedSpriteId = true;
      }
      textureIndex = 0;
    }
    const targetHeight = renderList[i + 4];
    const alpha = Math.max(0, Math.min(1, renderList[i + 5] ?? 1));
    const entityIndex = i / stride;

    let sprite = spritesByEntity[entityIndex];
    const tex = textures[textureIndex] ?? textures[0];
    if (!sprite) {
      sprite = new Sprite(tex);
      sprite.anchor.set(0.5, 0.5);
      spritesByEntity[entityIndex] = sprite;
      app.stage.addChild(sprite);
    }

    if (sprite.texture !== tex) {
      sprite.texture = tex;
    }
    const scale = targetHeight / sprite.texture.height;
    const flipX = spriteId === 7 ? -1 : 1;
    sprite.scale.set(scale * flipX, scale);

    sprite.x = x;
    sprite.y = y;
    sprite.rotation = rotation;
    sprite.alpha = alpha;
  }

  debugLayer.clear();
  debugLayer.strokeStyle = { width: 1, color: 0x00ff66, alpha: 0.8 };
  for (let i = 0; i + 5 < debugList.length; i += 6) {
    const kind = debugList[i];
    const x = debugList[i + 1];
    const y = debugList[i + 2];
    const a = debugList[i + 3];
    const b = debugList[i + 4];
    const rotation = debugList[i + 5];

    if (kind === 0) {
      debugLayer.circle(x, y, a);
      debugLayer.stroke();
    } else if (kind === 1) {
      debugLayer.rect(x - a, y - b, a * 2, b * 2);
      debugLayer.stroke();
    } else if (kind === 2) {
      const dx = Math.cos(rotation);
      const dy = Math.sin(rotation);
      const ax = x - dx * a;
      const ay = y - dy * a;
      const bx = x + dx * a;
      const by = y + dy * a;
      debugLayer.moveTo(ax, ay);
      debugLayer.lineTo(bx, by);
      debugLayer.stroke();
      debugLayer.circle(ax, ay, b);
      debugLayer.stroke();
      debugLayer.circle(bx, by, b);
      debugLayer.stroke();
    }
  }
});
