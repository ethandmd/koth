import { Application, Assets, Sprite, TextureStyle } from 'pixi.js';
import init, { Game, InputState } from '../pkg/koth_core.js';

const app = new Application();
await app.init({
  background: '#0b0b0b',
  resizeTo: window,
});

TextureStyle.defaultOptions.scaleMode = 'nearest';

document.getElementById('app').appendChild(app.canvas);

const spritePaths = [
  '/sprites/retro-triguy-stride1-sprite.png',
  '/sprites/retro-triguy-strike-sprite.png',
  '/sprites/retro-wedgeguy-stride-sprite.png',
  '/sprites/retro-wedgeguy-strike-sprite.png',
  '/sprites/retro-castle-sprite.png',
  '/sprites/retro-cannon-sprite.png',
  '/sprites/retro-crossbow-sprite.png',
];

const loadedTextures = await Assets.load(spritePaths);
const textures = spritePaths.map((path) => loadedTextures[path]);

await init();
const game = new Game();
const input = new InputState();

const spritesByEntity = [];

const setViewport = () => {
  game.set_viewport(app.canvas.width, app.canvas.height);
};
setViewport();
window.addEventListener('resize', () => {
  setViewport();
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
  input.pointer_x = pointer.x;
  input.pointer_y = pointer.y;
  input.pointer_down = pointer.down;

  const dt = ticker.deltaMS / 1000;
  game.tick(dt, input);

  const renderList = game.render_list();
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
      sprite.scale.set(0.05, 0.05);
      spritesByEntity[entityIndex] = sprite;
      app.stage.addChild(sprite);
    }

    if (spriteId === 4) {
      sprite.scale.set(0.1, 0.1);
    } else if (sprite.scale.x !== 0.05) {
      sprite.scale.set(0.05, 0.05);
    } else if (sprite.texture !== tex) {
      sprite.texture = tex;
    }

    sprite.x = x;
    sprite.y = y;
  }
});
