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
enum CharacterPhase {
    Inactive,
    Approaching,
    Falling,
}

#[derive(Clone, Copy)]
struct CharacterState {
    phase: CharacterPhase,
    respawn_timer: f32,
}

#[derive(Clone, Copy)]
struct ProjectileState {
    active: bool,
    cooldown: f32,
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
    offset: Vec2,
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
    score: u32,
    triguy_state: CharacterState,
    wedgeguy_state: CharacterState,
    arrow_state: ProjectileState,
    cannonball_state: ProjectileState,
    world: World,
    castle: Entity,
    cannon: Entity,
    crossbow: Entity,
    arrow: Entity,
    cannonball: Entity,
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
                    half_extents: Vec2::new(55.0, 70.0),
                },
                layer: 1,
                mask: 0,
                offset: Vec2::new(0.0, -18.0),
            },
        ));

        let cannon = world.spawn((
            Transform { pos: Vec2::ZERO },
            Renderable { sprite_id: 5 },
            Collider {
                shape: ColliderShape::Aabb {
                    half_extents: Vec2::new(40.0, 30.0),
                },
                layer: 1,
                mask: 0,
                offset: Vec2::ZERO,
            },
        ));

        let crossbow = world.spawn((
            Transform { pos: Vec2::ZERO },
            Renderable { sprite_id: 6 },
            Collider {
                shape: ColliderShape::Aabb {
                    half_extents: Vec2::new(40.0, 30.0),
                },
                layer: 1,
                mask: 0,
                offset: Vec2::ZERO,
            },
        ));

        let arrow = world.spawn((
            Transform { pos: Vec2::ZERO },
            Velocity { vel: Vec2::new(-260.0, 0.0) },
            Renderable { sprite_id: 7 },
            Collider {
                shape: ColliderShape::Circle { radius: 10.0 },
                layer: 1,
                mask: 0,
                offset: Vec2::ZERO,
            },
        ));

        let cannonball = world.spawn((
            Transform { pos: Vec2::ZERO },
            Velocity { vel: Vec2::new(200.0, 0.0) },
            Renderable { sprite_id: 8 },
            Collider {
                shape: ColliderShape::Circle { radius: 12.0 },
                layer: 1,
                mask: 0,
                offset: Vec2::ZERO,
            },
        ));

        let triguy = world.spawn((
            Transform { pos: Vec2::ZERO },
            Velocity { vel: Vec2::ZERO },
            Renderable { sprite_id: 0 },
            Collider {
                shape: ColliderShape::Circle { radius: 28.0 },
                layer: 2,
                mask: 1,
                offset: Vec2::ZERO,
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
                offset: Vec2::ZERO,
            },
        ));

        Game {
            time: 0.0,
            viewport_w: 0.0,
            viewport_h: 0.0,
            score: 0,
            triguy_state: CharacterState {
                phase: CharacterPhase::Inactive,
                respawn_timer: 0.25,
            },
            wedgeguy_state: CharacterState {
                phase: CharacterPhase::Inactive,
                respawn_timer: 0.75,
            },
            arrow_state: ProjectileState {
                active: false,
                cooldown: 0.0,
            },
            cannonball_state: ProjectileState {
                active: false,
                cooldown: 0.0,
            },
            world,
            castle,
            cannon,
            crossbow,
            arrow,
            cannonball,
            triguy,
            wedgeguy,
        }
    }

    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_w = width;
        self.viewport_h = height;
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn tick(&mut self, dt: f32, input: &InputState) {
        self.time += dt;

        if self.viewport_w <= 0.0 || self.viewport_h <= 0.0 {
            return;
        }

        let center_x = self.viewport_w * 0.5;
        let mut left_stop = center_x - 140.0;
        let mut right_stop = center_x + 140.0;
        let speed = 90.0;
        let fall_speed = 220.0;
        let arrow_speed = 420.0;
        let cannon_speed = 340.0;
        let arrow_gravity = 520.0;
        let cannon_gravity = 760.0;
        let arrow_cooldown = 0.35;
        let cannon_cooldown = 0.8;

        let center_y = self.viewport_h * 0.5;
        let ground_y = self.viewport_h * 0.75;
        let base = self.viewport_w.min(self.viewport_h);
        let castle_height = base * 0.25;
        let weapon_height = castle_height * 0.22;
        let character_height = castle_height * 0.35;
        let projectile_height = weapon_height * 0.35;

        let castle_base_he = Vec2::new(55.0, 70.0);
        let weapon_base_he = Vec2::new(40.0, 30.0);
        let castle_scale = castle_height / (castle_base_he.y * 2.0);
        let weapon_scale = weapon_height / (weapon_base_he.y * 2.0);
        let character_scale = character_height / 56.0;
        let character_offscreen = character_height * 3.0;
        let projectile_offscreen = projectile_height * 4.0;
        let hidden_pos = Vec2::new(-10000.0, -10000.0);

        let anim = (self.time * 6.0).floor() as i32 % 2;
        let triguy_sprite = if anim == 0 { 0 } else { 1 };
        let wedgeguy_sprite = if anim == 0 { 2 } else { 3 };

        if let (Ok(mut t), Ok(mut c_collider)) = (
            self.world.get::<&mut Transform>(self.castle),
            self.world.get::<&mut Collider>(self.castle),
        ) {
            let mut castle_y = center_y;
            let new_half_extents = castle_base_he * castle_scale;
            if let ColliderShape::Aabb { .. } = c_collider.shape {
                // Align castle bottom to the shared ground line.
                castle_y = ground_y - new_half_extents.y - c_collider.offset.y;
            }
            t.pos = Vec2::new(center_x, castle_y);
            c_collider.shape = ColliderShape::Aabb {
                half_extents: new_half_extents,
            };
        }
        if let Ok(mut c_collider) = self.world.get::<&mut Collider>(self.cannon) {
            c_collider.shape = ColliderShape::Aabb {
                half_extents: weapon_base_he * weapon_scale,
            };
        }
        if let Ok(mut c_collider) = self.world.get::<&mut Collider>(self.crossbow) {
            c_collider.shape = ColliderShape::Aabb {
                half_extents: weapon_base_he * weapon_scale,
            };
        }
        if let Ok(mut c_collider) = self.world.get::<&mut Collider>(self.triguy) {
            c_collider.shape = ColliderShape::Circle {
                radius: 28.0 * character_scale,
            };
        }
        if let Ok(mut c_collider) = self.world.get::<&mut Collider>(self.wedgeguy) {
            c_collider.shape = ColliderShape::Circle {
                radius: 28.0 * character_scale,
            };
        }
        if let Ok(mut c_collider) = self.world.get::<&mut Collider>(self.arrow) {
            c_collider.shape = ColliderShape::Circle {
                radius: projectile_height * 0.4,
            };
        }
        if let Ok(mut c_collider) = self.world.get::<&mut Collider>(self.cannonball) {
            c_collider.shape = ColliderShape::Circle {
                radius: projectile_height * 0.45,
            };
        }
        if let (Ok(castle), Ok(c_collider)) = (
            self.world.get::<&Transform>(self.castle),
            self.world.get::<&Collider>(self.castle),
        ) {
            if let ColliderShape::Aabb { half_extents: c_he } = c_collider.shape {
                let castle_center = castle.pos + c_collider.offset;
                left_stop = castle_center.x - c_he.x;
                right_stop = castle_center.x + c_he.x;
            }
        }
        let mut cannon_target = None;
        let mut cross_target = None;
        if let (Ok(castle), Ok(c_collider)) = (
            self.world.get::<&Transform>(self.castle),
            self.world.get::<&Collider>(self.castle),
        ) {
            if let ColliderShape::Aabb { half_extents: c_he } = c_collider.shape {
                let castle_top_right = castle.pos + c_collider.offset + Vec2::new(c_he.x, -c_he.y);
                let castle_top_left = castle.pos + c_collider.offset + Vec2::new(-c_he.x, -c_he.y);

                if let Ok(cannon_collider) = self.world.get::<&Collider>(self.cannon) {
                    if let ColliderShape::Aabb { half_extents: a_he } = cannon_collider.shape {
                        let cannon_center =
                            castle_top_right - cannon_collider.offset + Vec2::new(-a_he.x, -a_he.y);
                        cannon_target = Some(cannon_center);
                    }
                }

                if let Ok(cross_collider) = self.world.get::<&Collider>(self.crossbow) {
                    if let ColliderShape::Aabb { half_extents: a_he } = cross_collider.shape {
                        let cross_center =
                            castle_top_left - cross_collider.offset + Vec2::new(a_he.x, -a_he.y);
                        cross_target = Some(cross_center);
                    }
                }
            }
        }

        if let (Some(pos), Ok(mut t)) = (cannon_target, self.world.get::<&mut Transform>(self.cannon)) {
            t.pos = pos;
        }
        if let (Some(pos), Ok(mut t)) = (cross_target, self.world.get::<&mut Transform>(self.crossbow)) {
            t.pos = pos;
        }

        let mut crossbow_spawn = None;
        if let (Ok(t), Ok(c)) = (
            self.world.get::<&Transform>(self.crossbow),
            self.world.get::<&Collider>(self.crossbow),
        ) {
            let mut spawn = t.pos + c.offset;
            if let ColliderShape::Aabb { half_extents } = c.shape {
                spawn += Vec2::new(-half_extents.x, -weapon_height * 0.1);
            }
            crossbow_spawn = Some(spawn);
        }

        let mut cannon_spawn = None;
        if let (Ok(t), Ok(c)) = (
            self.world.get::<&Transform>(self.cannon),
            self.world.get::<&Collider>(self.cannon),
        ) {
            let mut spawn = t.pos + c.offset;
            if let ColliderShape::Aabb { half_extents } = c.shape {
                spawn += Vec2::new(half_extents.x, -weapon_height * 0.1);
            }
            cannon_spawn = Some(spawn);
        }

        self.arrow_state.cooldown = (self.arrow_state.cooldown - dt).max(0.0);
        self.cannonball_state.cooldown = (self.cannonball_state.cooldown - dt).max(0.0);

        let wants_fire = input.pointer_down;
        let aim_pos = Vec2::new(input.pointer_x, input.pointer_y);
        if wants_fire {
            if aim_pos.x <= center_x {
                if self.arrow_state.cooldown <= 0.0 && !self.arrow_state.active {
                    if let (Some(spawn), Ok(mut proj_t), Ok(mut v)) = (
                        crossbow_spawn,
                        self.world.get::<&mut Transform>(self.arrow),
                        self.world.get::<&mut Velocity>(self.arrow),
                    ) {
                        let mut dir = aim_pos - spawn;
                        if dir.length_squared() < 1.0 {
                            dir = Vec2::new(-1.0, 0.0);
                        }
                        dir = dir.normalize();
                        proj_t.pos = spawn;
                        v.vel = dir * arrow_speed;
                        self.arrow_state.active = true;
                        self.arrow_state.cooldown = arrow_cooldown;
                    }
                }
            } else if self.cannonball_state.cooldown <= 0.0 && !self.cannonball_state.active {
                if let (Some(spawn), Ok(mut proj_t), Ok(mut v)) = (
                    cannon_spawn,
                    self.world.get::<&mut Transform>(self.cannonball),
                    self.world.get::<&mut Velocity>(self.cannonball),
                ) {
                    let mut dir = aim_pos - spawn;
                    if dir.length_squared() < 1.0 {
                        dir = Vec2::new(1.0, 0.0);
                    }
                    dir = dir.normalize();
                    proj_t.pos = spawn;
                    v.vel = dir * cannon_speed;
                    self.cannonball_state.active = true;
                    self.cannonball_state.cooldown = cannon_cooldown;
                }
            }
        }

        if let (Ok(mut proj_t), Ok(mut v)) = (
            self.world.get::<&mut Transform>(self.arrow),
            self.world.get::<&mut Velocity>(self.arrow),
        ) {
            if self.arrow_state.active {
                v.vel.y += arrow_gravity * dt;
                proj_t.pos += v.vel * dt;
                let offscreen = proj_t.pos.x < -projectile_offscreen
                    || proj_t.pos.x > self.viewport_w + projectile_offscreen
                    || proj_t.pos.y > self.viewport_h + projectile_offscreen
                    || proj_t.pos.y < -projectile_offscreen;
                if offscreen {
                    self.arrow_state.active = false;
                    proj_t.pos = hidden_pos;
                }
            } else if proj_t.pos != hidden_pos {
                proj_t.pos = hidden_pos;
            }
        }

        if let (Ok(mut proj_t), Ok(mut v)) = (
            self.world.get::<&mut Transform>(self.cannonball),
            self.world.get::<&mut Velocity>(self.cannonball),
        ) {
            if self.cannonball_state.active {
                v.vel.y += cannon_gravity * dt;
                proj_t.pos += v.vel * dt;
                let offscreen = proj_t.pos.x < -projectile_offscreen
                    || proj_t.pos.x > self.viewport_w + projectile_offscreen
                    || proj_t.pos.y > self.viewport_h + projectile_offscreen
                    || proj_t.pos.y < -projectile_offscreen;
                if offscreen {
                    self.cannonball_state.active = false;
                    proj_t.pos = hidden_pos;
                }
            } else if proj_t.pos != hidden_pos {
                proj_t.pos = hidden_pos;
            }
        }

        self.triguy_state.respawn_timer -= dt;
        if let (Ok(mut t), Ok(mut v)) = (
            self.world.get::<&mut Transform>(self.triguy),
            self.world.get::<&mut Velocity>(self.triguy),
        ) {
            match self.triguy_state.phase {
                CharacterPhase::Inactive => {
                    if self.triguy_state.respawn_timer <= 0.0 {
                        t.pos = Vec2::new(-character_offscreen, ground_y);
                        v.vel = Vec2::new(speed, 0.0);
                        self.triguy_state.phase = CharacterPhase::Approaching;
                    } else {
                        t.pos = Vec2::new(-character_offscreen * 4.0, self.viewport_h + character_offscreen * 4.0);
                    }
                }
                CharacterPhase::Approaching => {
                    t.pos += v.vel * dt;
                    if t.pos.x >= left_stop {
                        t.pos.x = left_stop;
                        v.vel = Vec2::new(0.0, fall_speed);
                        self.triguy_state.phase = CharacterPhase::Falling;
                    }
                }
                CharacterPhase::Falling => {
                    t.pos += v.vel * dt;
                    if t.pos.y > self.viewport_h + character_offscreen {
                        self.triguy_state.phase = CharacterPhase::Inactive;
                        self.triguy_state.respawn_timer = 1.0;
                    }
                }
            }
        }
        if let Ok(mut r) = self.world.get::<&mut Renderable>(self.triguy) {
            r.sprite_id = triguy_sprite;
        }

        self.wedgeguy_state.respawn_timer -= dt;
        if let (Ok(mut t), Ok(mut v)) = (
            self.world.get::<&mut Transform>(self.wedgeguy),
            self.world.get::<&mut Velocity>(self.wedgeguy),
        ) {
            match self.wedgeguy_state.phase {
                CharacterPhase::Inactive => {
                    if self.wedgeguy_state.respawn_timer <= 0.0 {
                        t.pos = Vec2::new(self.viewport_w + character_offscreen, ground_y);
                        v.vel = Vec2::new(-speed, 0.0);
                        self.wedgeguy_state.phase = CharacterPhase::Approaching;
                    } else {
                        t.pos = Vec2::new(-character_offscreen * 4.0, self.viewport_h + character_offscreen * 4.0);
                    }
                }
                CharacterPhase::Approaching => {
                    t.pos += v.vel * dt;
                    if t.pos.x <= right_stop {
                        t.pos.x = right_stop;
                        v.vel = Vec2::new(0.0, fall_speed);
                        self.wedgeguy_state.phase = CharacterPhase::Falling;
                    }
                }
                CharacterPhase::Falling => {
                    t.pos += v.vel * dt;
                    if t.pos.y > self.viewport_h + character_offscreen {
                        self.wedgeguy_state.phase = CharacterPhase::Inactive;
                        self.wedgeguy_state.respawn_timer = 1.0;
                    }
                }
            }
        }
        if let Ok(mut r) = self.world.get::<&mut Renderable>(self.wedgeguy) {
            r.sprite_id = wedgeguy_sprite;
        }

        self.resolve_projectile_hits();
    }

    pub fn render_list(&self) -> Vec<f32> {
        // Packed: [x, y, rotation, sprite_id]. Exposed to JS as a Float32Array.
        // Sprite IDs must match the JS spritePaths order.
        let mut out = Vec::with_capacity(7 * 4);
        for entity in [
            self.castle,
            self.cannon,
            self.crossbow,
            self.arrow,
            self.cannonball,
            self.triguy,
            self.wedgeguy,
        ] {
            if let (Ok(t), Ok(r)) = (
                self.world.get::<&Transform>(entity),
                self.world.get::<&Renderable>(entity),
            ) {
                let rotation = if entity == self.arrow && self.arrow_state.active {
                    self.world
                        .get::<&Velocity>(self.arrow)
                        .map(|v| v.vel.y.atan2(v.vel.x))
                        .unwrap_or(0.0)
                } else if entity == self.cannonball && self.cannonball_state.active {
                    self.world
                        .get::<&Velocity>(self.cannonball)
                        .map(|v| v.vel.y.atan2(v.vel.x))
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                out.push(t.pos.x);
                out.push(t.pos.y);
                out.push(rotation);
                out.push(r.sprite_id as f32);
            }
        }
        out
    }

    pub fn debug_list(&self) -> Vec<f32> {
        // Packed: [kind, x, y, a, b]
        // kind: 0 = circle (a=radius), 1 = aabb (a=half_w, b=half_h)
        let mut out = Vec::new();
        for (_, (t, c)) in self.world.query::<(&Transform, &Collider)>().iter() {
            let center = t.pos + c.offset;
            match c.shape {
                ColliderShape::Circle { radius } => {
                    out.push(0.0);
                    out.push(center.x);
                    out.push(center.y);
                    out.push(radius);
                    out.push(0.0);
                }
                ColliderShape::Aabb { half_extents } => {
                    out.push(1.0);
                    out.push(center.x);
                    out.push(center.y);
                    out.push(half_extents.x);
                    out.push(half_extents.y);
                }
            }
        }
        out
    }
}

impl Game {
    fn resolve_projectile_hits(&mut self) {
        let mut arrow_hit = false;
        let mut cannon_hit = false;
        let mut score_add = 0u32;

        let arrow_data = if self.arrow_state.active {
            if let (Ok(t), Ok(c)) = (
                self.world.get::<&Transform>(self.arrow),
                self.world.get::<&Collider>(self.arrow),
            ) {
                if let ColliderShape::Circle { radius } = c.shape {
                    Some((t.pos + c.offset, radius))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let cannon_data = if self.cannonball_state.active {
            if let (Ok(t), Ok(c)) = (
                self.world.get::<&Transform>(self.cannonball),
                self.world.get::<&Collider>(self.cannonball),
            ) {
                if let ColliderShape::Circle { radius } = c.shape {
                    Some((t.pos + c.offset, radius))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some((proj_pos, proj_r)) = arrow_data {
            if try_hit_character(
                &mut self.world,
                self.triguy,
                proj_pos,
                proj_r,
                &mut self.triguy_state,
            ) {
                arrow_hit = true;
                score_add = score_add.saturating_add(1);
            }
            if try_hit_character(
                &mut self.world,
                self.wedgeguy,
                proj_pos,
                proj_r,
                &mut self.wedgeguy_state,
            )
            {
                arrow_hit = true;
                score_add = score_add.saturating_add(1);
            }
        }

        if let Some((proj_pos, proj_r)) = cannon_data {
            if try_hit_character(
                &mut self.world,
                self.triguy,
                proj_pos,
                proj_r,
                &mut self.triguy_state,
            ) {
                cannon_hit = true;
                score_add = score_add.saturating_add(1);
            }
            if try_hit_character(
                &mut self.world,
                self.wedgeguy,
                proj_pos,
                proj_r,
                &mut self.wedgeguy_state,
            )
            {
                cannon_hit = true;
                score_add = score_add.saturating_add(1);
            }
        }

        if arrow_hit {
            self.arrow_state.active = false;
            if let Ok(mut t) = self.world.get::<&mut Transform>(self.arrow) {
                t.pos = Vec2::new(-10000.0, -10000.0);
            }
        }

        if cannon_hit {
            self.cannonball_state.active = false;
            if let Ok(mut t) = self.world.get::<&mut Transform>(self.cannonball) {
                t.pos = Vec2::new(-10000.0, -10000.0);
            }
        }

        if score_add > 0 {
            self.score = self.score.saturating_add(score_add);
        }
    }
}

fn circle_hit(a_pos: Vec2, a_r: f32, b_pos: Vec2, b_r: f32) -> bool {
    let delta = a_pos - b_pos;
    let radius = a_r + b_r;
    delta.length_squared() <= radius * radius
}

fn try_hit_character(
    world: &mut World,
    entity: Entity,
    proj_pos: Vec2,
    proj_r: f32,
    state: &mut CharacterState,
) -> bool {
    if let CharacterPhase::Inactive = state.phase {
        return false;
    }
    let (pos, radius) = if let (Ok(t), Ok(c)) =
        (world.get::<&Transform>(entity), world.get::<&Collider>(entity))
    {
        if let ColliderShape::Circle { radius } = c.shape {
            (t.pos + c.offset, radius)
        } else {
            return false;
        }
    } else {
        return false;
    };
    if circle_hit(proj_pos, proj_r, pos, radius) {
        state.phase = CharacterPhase::Inactive;
        state.respawn_timer = 1.0;
        if let Ok(mut t) = world.get::<&mut Transform>(entity) {
            t.pos = Vec2::new(-10000.0, -10000.0);
        }
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_does_not_panic_with_viewport() {
        let mut game = Game::new();
        game.set_viewport(800.0, 600.0);
        let input = InputState::new();
        game.tick(1.0 / 60.0, &input);
        game.tick(1.0 / 60.0, &input);
        let list = game.render_list();
        assert!(!list.is_empty());
        let dbg = game.debug_list();
        assert!(!dbg.is_empty());
    }
}
