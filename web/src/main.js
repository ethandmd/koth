import { Application, Assets, Graphics, Sprite, TextureStyle } from 'pixi.js';
import init, { Game, InputState } from '../pkg/koth_core.js';

const app = new Application();
await app.init({
  background: '#0b0b0b',
  resizeTo: window,
  preference: 'webgl',
  powerPreference: 'high-performance',
});

TextureStyle.defaultOptions.scaleMode = 'nearest';

document.getElementById('app').appendChild(app.canvas);

app.canvas.addEventListener('webglcontextlost', (event) => {
  event.preventDefault();
  console.error('WebGL context lost', event);
});
app.canvas.addEventListener('webglcontextrestored', () => {
  console.warn('WebGL context restored');
});

console.info('Pixi renderer:', app.renderer?.constructor?.name ?? 'unknown');

const spritePaths = [
  '/sprites/retro-triguy-stride1-sprite.png',
  '/sprites/retro-triguy-strike-sprite.png',
  '/sprites/retro-wedgeguy-stride-sprite.png',
  '/sprites/retro-wedgeguy-strike-sprite.png',
  '/sprites/retro-castle-sprite.png',
  '/sprites/retro-cannon-sprite.png',
  '/sprites/retro-crossbow-sprite.png',
  '/sprites/retro-arrow-sprite.png',
  '/sprites/retro-cannonball-sprite.png',
];

const loadedTextures = await Assets.load(spritePaths);
const textures = spritePaths.map((path) => loadedTextures[path]);

await init();
const game = new Game();
const input = new InputState();
const scoreEl = document.getElementById('score');
let lastScore = -1;

const spritesByEntity = [];
const debugLayer = new Graphics();
app.stage.addChild(debugLayer);

let pendingViewport = { w: app.canvas.width, h: app.canvas.height };
window.addEventListener('resize', () => {
  pendingViewport = { w: app.canvas.width, h: app.canvas.height };
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
  if (pendingViewport) {
    game.set_viewport(pendingViewport.w, pendingViewport.h);
    pendingViewport = null;
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

  for (let i = 0; i + 2 < renderList.length; i += 3) {
    const x = renderList[i];
    const y = renderList[i + 1];
    const spriteId = renderList[i + 2];
    const entityIndex = i / 3;

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
    sprite.scale.set(scale, scale);

    sprite.x = x;
    sprite.y = y;
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
