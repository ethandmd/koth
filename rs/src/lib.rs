use wasm_bindgen::prelude::*;
use glam::Vec2;
use hecs::{Entity, World};

#[derive(Clone, Copy)]
struct Transform {
    pos: Vec2,
}

#[derive(Clone, Copy)]
struct Velocity {
    vel: Vec2,
}

#[derive(Clone, Copy)]
struct Renderable {
    sprite_id: u16,
}

#[derive(Clone, Copy)]
enum ColliderShape {
    Circle { radius: f32 },
    Aabb { half_extents: Vec2 },
}

#[derive(Clone, Copy)]
struct Collider {
    shape: ColliderShape,
    layer: u32,
    mask: u32,
}

#[wasm_bindgen]
pub struct InputState {
    pub pointer_x: f32,
    pub pointer_y: f32,
    pub pointer_down: bool,
}

#[wasm_bindgen]
impl InputState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> InputState {
        InputState {
            pointer_x: 0.0,
            pointer_y: 0.0,
            pointer_down: false,
        }
    }
}

#[wasm_bindgen]
pub struct Game {
    time: f32,
    viewport_w: f32,
    viewport_h: f32,
    triguy_x: f32,
    wedgeguy_x: f32,
    world: World,
    castle: Entity,
    cannon: Entity,
    crossbow: Entity,
    triguy: Entity,
    wedgeguy: Entity,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Game {
        let mut world = World::new();

        let castle = world.spawn((
            Transform { pos: Vec2::ZERO },
            Renderable { sprite_id: 4 },
            Collider {
                shape: ColliderShape::Aabb {
                    half_extents: Vec2::new(80.0, 80.0),
                },
                layer: 1,
                mask: 0,
            },
        ));

        let cannon = world.spawn((
            Transform { pos: Vec2::ZERO },
            Renderable { sprite_id: 5 },
        ));

        let crossbow = world.spawn((
            Transform { pos: Vec2::ZERO },
            Renderable { sprite_id: 6 },
        ));

        let triguy = world.spawn((
            Transform { pos: Vec2::ZERO },
            Velocity { vel: Vec2::ZERO },
            Renderable { sprite_id: 0 },
            Collider {
                shape: ColliderShape::Circle { radius: 28.0 },
                layer: 2,
                mask: 1,
            },
        ));

        let wedgeguy = world.spawn((
            Transform { pos: Vec2::ZERO },
            Velocity { vel: Vec2::ZERO },
            Renderable { sprite_id: 2 },
            Collider {
                shape: ColliderShape::Circle { radius: 28.0 },
                layer: 2,
                mask: 1,
            },
        ));

        Game {
            time: 0.0,
            viewport_w: 0.0,
            viewport_h: 0.0,
            triguy_x: -200.0,
            wedgeguy_x: 200.0,
            world,
            castle,
            cannon,
            crossbow,
            triguy,
            wedgeguy,
        }
    }

    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_w = width;
        self.viewport_h = height;

        if self.triguy_x == -200.0 && self.wedgeguy_x == 200.0 {
            self.triguy_x = -200.0;
            self.wedgeguy_x = width + 200.0;
        }
    }

    pub fn tick(&mut self, dt: f32, _input: &InputState) {
        self.time += dt;

        if self.viewport_w <= 0.0 || self.viewport_h <= 0.0 {
            return;
        }

        let center_x = self.viewport_w * 0.5;
        let left_stop = center_x - 140.0;
        let right_stop = center_x + 140.0;
        let speed = 120.0;

        if self.triguy_x < left_stop {
            self.triguy_x = (self.triguy_x + speed * dt).min(left_stop);
        }
        if self.wedgeguy_x > right_stop {
            self.wedgeguy_x = (self.wedgeguy_x - speed * dt).max(right_stop);
        }

        let center_y = self.viewport_h * 0.5;
        let ground_y = self.viewport_h * 0.68;
        let tower_offset_x = 220.0;
        let tower_y = center_y - 40.0;

        let anim = (self.time * 6.0).floor() as i32 % 2;
        let triguy_sprite = if anim == 0 { 0 } else { 1 };
        let wedgeguy_sprite = if anim == 0 { 2 } else { 3 };

        if let Ok(mut t) = self.world.get::<&mut Transform>(self.castle) {
            t.pos = Vec2::new(center_x, center_y);
        }
        if let Ok(mut t) = self.world.get::<&mut Transform>(self.cannon) {
            t.pos = Vec2::new(center_x + tower_offset_x, tower_y);
        }
        if let Ok(mut t) = self.world.get::<&mut Transform>(self.crossbow) {
            t.pos = Vec2::new(center_x - tower_offset_x, tower_y);
        }

        if let Ok(mut t) = self.world.get::<&mut Transform>(self.triguy) {
            t.pos = Vec2::new(self.triguy_x, ground_y);
        }
        if let Ok(mut r) = self.world.get::<&mut Renderable>(self.triguy) {
            r.sprite_id = triguy_sprite;
        }

        if let Ok(mut t) = self.world.get::<&mut Transform>(self.wedgeguy) {
            t.pos = Vec2::new(self.wedgeguy_x, ground_y);
        }
        if let Ok(mut r) = self.world.get::<&mut Renderable>(self.wedgeguy) {
            r.sprite_id = wedgeguy_sprite;
        }
    }

    pub fn render_list(&self) -> Vec<f32> {
        // Packed: [x, y, sprite_id]. Exposed to JS as a Float32Array.
        // Sprite IDs must match the JS spritePaths order.
        let mut out = Vec::with_capacity(5 * 3);
        for entity in [
            self.castle,
            self.cannon,
            self.crossbow,
            self.triguy,
            self.wedgeguy,
        ] {
            if let (Ok(t), Ok(r)) = (
                self.world.get::<&Transform>(entity),
                self.world.get::<&Renderable>(entity),
            ) {
                out.push(t.pos.x);
                out.push(t.pos.y);
                out.push(r.sprite_id as f32);
            }
        }
        out
    }
}
