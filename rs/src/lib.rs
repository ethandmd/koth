use wasm_bindgen::prelude::*;
use js_sys::Float32Array;
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
    Attacking,
}

#[derive(Clone, Copy)]
struct CharacterState {
    phase: CharacterPhase,
    respawn_timer: f32,
}

#[derive(Clone, Copy)]
struct ProjectileSlot {
    entity: Entity,
    active: bool,
}

#[derive(Clone, Copy)]
enum ArrowState {
    Inactive,
    Flying,
    Grounded {
        fade_timer: f32,
        alpha: f32,
        rotation: f32,
    },
}

#[derive(Clone, Copy)]
struct ArrowSlot {
    entity: Entity,
    state: ArrowState,
}

#[derive(Clone, Copy)]
struct ExplosionSlot {
    entity: Entity,
    active: bool,
    stage: u8,
    timer: f32,
}

#[derive(Clone, Copy)]
struct Renderable {
    sprite_id: u16,
}

#[derive(Clone, Copy)]
struct SpriteBounds {
    sprite_w: f32,
    sprite_h: f32,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    valid: bool,
}

impl Default for SpriteBounds {
    fn default() -> Self {
        SpriteBounds {
            sprite_w: 0.0,
            sprite_h: 0.0,
            min_x: 0.0,
            min_y: 0.0,
            max_x: 0.0,
            max_y: 0.0,
            valid: false,
        }
    }
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
    wall_integrity: f32,
    game_over: bool,
    triguy_state: CharacterState,
    wedgeguy_state: CharacterState,
    crossbow_cooldown: f32,
    cannon_cooldown: f32,
    world: World,
    castle: Entity,
    cannon: Entity,
    crossbow: Entity,
    arrow_projectiles: Vec<ArrowSlot>,
    cannon_projectiles: Vec<ProjectileSlot>,
    explosions: Vec<ExplosionSlot>,
    triguy: Entity,
    wedgeguy: Entity,
    castle_sprite_bounds: SpriteBounds,
    render_buf: Vec<f32>,
    debug_buf: Vec<f32>,
    sprite_heights_cache: Vec<f32>,
}

const ARROW_POOL_SIZE: usize = 6;
const CANNONBALL_POOL_SIZE: usize = 4;
const EXPLOSION_POOL_SIZE: usize = 6;
const WALL_INTEGRITY_MAX: f32 = 1.0;
const WALL_DAMAGE_PER_SECOND: f32 = 0.04;
const GROUND_Y_RATIO: f32 = 0.75;
const CASTLE_GROUND_SINK_RATIO: f32 = 0.10;
const EXPLOSION_FRAME_TIME: f32 = 0.05;
const ARROW_FADE_INTERVAL: f32 = 0.05;
const ARROW_FADE_STEP: f32 = 0.2;
const ARROW_CAPSULE_HALF_RATIO: f32 = 0.55;
const ARROW_CAPSULE_RADIUS_RATIO: f32 = 0.18;
const EXPLOSION_GROW_STAGES: u8 = 5;
const EXPLOSION_SHRINK_STAGES: u8 = 2;
const EXPLOSION_STAGE_COUNT: u8 = EXPLOSION_GROW_STAGES + EXPLOSION_SHRINK_STAGES;
const SPRITE_EXPLOSION_BASE: u16 = 9;
const BASE_SPRITE_COUNT: usize = 9;
const MAX_ENTITIES: usize =
    5 + ARROW_POOL_SIZE + CANNONBALL_POOL_SIZE + EXPLOSION_POOL_SIZE;
const SPRITE_PATHS: [&str; BASE_SPRITE_COUNT] = [
    "sprites/retro-triguy-stride1-sprite.png",
    "sprites/retro-triguy-strike-sprite.png",
    "sprites/retro-wedgeguy-stride-sprite.png",
    "sprites/retro-wedgeguy-strike-sprite.png",
    "sprites/retro-castle-sprite.png",
    "sprites/retro-cannon-sprite.png",
    "sprites/retro-crossbow-sprite.png",
    "sprites/retro-arrow-sprite.png",
    "sprites/retro-cannonball-sprite.png",
];

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

        let mut arrow_projectiles = Vec::with_capacity(ARROW_POOL_SIZE);
        for _ in 0..ARROW_POOL_SIZE {
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
            arrow_projectiles.push(ArrowSlot {
                entity: arrow,
                state: ArrowState::Inactive,
            });
        }

        let mut cannon_projectiles = Vec::with_capacity(CANNONBALL_POOL_SIZE);
        for _ in 0..CANNONBALL_POOL_SIZE {
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
            cannon_projectiles.push(ProjectileSlot {
                entity: cannonball,
                active: false,
            });
        }

        let mut explosions = Vec::with_capacity(EXPLOSION_POOL_SIZE);
        for _ in 0..EXPLOSION_POOL_SIZE {
            let explosion = world.spawn((
                Transform { pos: Vec2::ZERO },
                Renderable {
                    sprite_id: SPRITE_EXPLOSION_BASE,
                },
                Collider {
                    shape: ColliderShape::Circle { radius: 10.0 },
                    layer: 1,
                    mask: 0,
                    offset: Vec2::ZERO,
                },
            ));
            explosions.push(ExplosionSlot {
                entity: explosion,
                active: false,
                stage: 0,
                timer: 0.0,
            });
        }

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
            wall_integrity: WALL_INTEGRITY_MAX,
            game_over: false,
            triguy_state: CharacterState {
                phase: CharacterPhase::Inactive,
                respawn_timer: 0.25,
            },
            wedgeguy_state: CharacterState {
                phase: CharacterPhase::Inactive,
                respawn_timer: 0.75,
            },
            crossbow_cooldown: 0.0,
            cannon_cooldown: 0.0,
            world,
            castle,
            cannon,
            crossbow,
            arrow_projectiles,
            cannon_projectiles,
            explosions,
            triguy,
            wedgeguy,
            castle_sprite_bounds: SpriteBounds::default(),
            render_buf: Vec::with_capacity(MAX_ENTITIES * 5),
            debug_buf: Vec::new(),
            sprite_heights_cache: vec![0.0; BASE_SPRITE_COUNT + EXPLOSION_STAGE_COUNT as usize],
        }
    }

    pub fn explosion_sprite_base(&self) -> u16 {
        SPRITE_EXPLOSION_BASE
    }

    pub fn explosion_scales(&self) -> Vec<f32> {
        (0..EXPLOSION_STAGE_COUNT)
            .map(explosion_scale)
            .collect()
    }

    pub fn sprite_paths(&self) -> Vec<String> {
        let mut out: Vec<String> = SPRITE_PATHS.iter().map(|path| (*path).to_string()).collect();
        for _ in 0..EXPLOSION_STAGE_COUNT {
            out.push("sprites/explosion-sprite.png".to_string());
        }
        out
    }

    pub fn sprite_heights(&self) -> Vec<f32> {
        self.sprite_heights_cache.clone()
    }

    pub fn set_viewport(&mut self, width: f32, height: f32) {
        if (self.viewport_w - width).abs() < f32::EPSILON
            && (self.viewport_h - height).abs() < f32::EPSILON
        {
            return;
        }
        self.viewport_w = width;
        self.viewport_h = height;
        self.sprite_heights_cache = compute_sprite_heights(self.viewport_w, self.viewport_h);
    }

    pub fn ground_y(&self) -> f32 {
        self.viewport_h * GROUND_Y_RATIO
    }

    pub fn set_castle_sprite_bounds(
        &mut self,
        sprite_w: f32,
        sprite_h: f32,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
    ) {
        let valid = sprite_w > 0.0 && sprite_h > 0.0 && max_x > min_x && max_y > min_y;
        self.castle_sprite_bounds = SpriteBounds {
            sprite_w,
            sprite_h,
            min_x,
            min_y,
            max_x,
            max_y,
            valid,
        };
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn wall_integrity(&self) -> f32 {
        self.wall_integrity
    }

    pub fn game_over(&self) -> bool {
        self.game_over
    }

    pub fn restart(&mut self) {
        self.time = 0.0;
        self.score = 0;
        self.wall_integrity = WALL_INTEGRITY_MAX;
        self.game_over = false;
        self.triguy_state = CharacterState {
            phase: CharacterPhase::Inactive,
            respawn_timer: 0.25,
        };
        self.wedgeguy_state = CharacterState {
            phase: CharacterPhase::Inactive,
            respawn_timer: 0.75,
        };
        self.crossbow_cooldown = 0.0;
        self.cannon_cooldown = 0.0;

        let hidden_pos = Vec2::new(-10000.0, -10000.0);
        for entity in [
            self.triguy,
            self.wedgeguy,
            self.castle,
            self.cannon,
            self.crossbow,
        ] {
            if let Ok(mut t) = self.world.get::<&mut Transform>(entity) {
                t.pos = hidden_pos;
            }
        }
        for slot in &mut self.arrow_projectiles {
            slot.state = ArrowState::Inactive;
            if let Ok(mut t) = self.world.get::<&mut Transform>(slot.entity) {
                t.pos = hidden_pos;
            }
            if let Ok(mut v) = self.world.get::<&mut Velocity>(slot.entity) {
                v.vel = Vec2::ZERO;
            }
        }
        for slot in &mut self.cannon_projectiles {
            slot.active = false;
            if let Ok(mut t) = self.world.get::<&mut Transform>(slot.entity) {
                t.pos = hidden_pos;
            }
        }
        for slot in &mut self.explosions {
            slot.active = false;
            slot.stage = 0;
            slot.timer = 0.0;
            if let Ok(mut t) = self.world.get::<&mut Transform>(slot.entity) {
                t.pos = hidden_pos;
            }
        }
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
        let arrow_speed = 420.0;
        let cannon_speed = 340.0;
        let arrow_gravity = 520.0;
        let cannon_gravity = 760.0;
        let arrow_cadence = 0.15;
        let cannon_cadence = 1.00;

        let center_y = self.viewport_h * 0.5;
        let ground_y = self.viewport_h * GROUND_Y_RATIO;
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
        let ground_impact_y = ground_y + castle_height * CASTLE_GROUND_SINK_RATIO;

        let anim = (self.time * 6.0).floor() as i32 % 2;
        let triguy_sprite = if anim == 0 { 0 } else { 1 };
        let wedgeguy_sprite = if anim == 0 { 2 } else { 3 };

        if let (Ok(mut t), Ok(mut c_collider)) = (
            self.world.get::<&mut Transform>(self.castle),
            self.world.get::<&mut Collider>(self.castle),
        ) {
            let mut castle_y = center_y;
            let mut new_half_extents = castle_base_he * castle_scale;
            let mut new_offset = c_collider.offset;
            if self.castle_sprite_bounds.valid {
                let sprite = self.castle_sprite_bounds;
                let sprite_scale = castle_height / sprite.sprite_h;
                let content_w = sprite.max_x - sprite.min_x;
                let content_h = sprite.max_y - sprite.min_y;
                new_half_extents = Vec2::new(content_w * 0.5 * sprite_scale, content_h * 0.5 * sprite_scale);
                let content_center = Vec2::new(
                    (sprite.min_x + sprite.max_x) * 0.5,
                    (sprite.min_y + sprite.max_y) * 0.5,
                );
                let sprite_center = Vec2::new(sprite.sprite_w * 0.5, sprite.sprite_h * 0.5);
                let offset_px = content_center - sprite_center;
                new_offset = offset_px * sprite_scale;
            }
            if let ColliderShape::Aabb { .. } = c_collider.shape {
                // Align castle bottom to the shared ground line.
                let sink = castle_height * CASTLE_GROUND_SINK_RATIO;
                castle_y = ground_y - new_half_extents.y - new_offset.y + sink;
            }
            t.pos = Vec2::new(center_x, castle_y);
            c_collider.shape = ColliderShape::Aabb {
                half_extents: new_half_extents,
            };
            c_collider.offset = new_offset;
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
        for slot in &self.arrow_projectiles {
            if let Ok(mut c_collider) = self.world.get::<&mut Collider>(slot.entity) {
                c_collider.shape = ColliderShape::Circle {
                    radius: projectile_height * 0.4,
                };
            }
        }
        for slot in &self.cannon_projectiles {
            if let Ok(mut c_collider) = self.world.get::<&mut Collider>(slot.entity) {
                c_collider.shape = ColliderShape::Circle {
                    radius: projectile_height * 0.45,
                };
            }
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
        if self.game_over {
            return;
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

        self.crossbow_cooldown = (self.crossbow_cooldown - dt).max(0.0);
        self.cannon_cooldown = (self.cannon_cooldown - dt).max(0.0);

        let wants_fire = input.pointer_down;
        let aim_pos = Vec2::new(input.pointer_x, input.pointer_y);
        if wants_fire {
            if aim_pos.x <= center_x {
                if self.crossbow_cooldown <= 0.0 {
                    if let Some(slot) = self
                        .arrow_projectiles
                        .iter_mut()
                        .find(|s| matches!(s.state, ArrowState::Inactive))
                    {
                        if let (Some(spawn), Ok(mut proj_t), Ok(mut v)) = (
                            crossbow_spawn,
                            self.world.get::<&mut Transform>(slot.entity),
                            self.world.get::<&mut Velocity>(slot.entity),
                        ) {
                            let mut dir = aim_pos - spawn;
                            if dir.length_squared() < 1.0 {
                                dir = Vec2::new(-1.0, 0.0);
                            }
                            dir = dir.normalize();
                            proj_t.pos = spawn;
                            v.vel = dir * arrow_speed;
                            slot.state = ArrowState::Flying;
                            self.crossbow_cooldown = arrow_cadence;
                        }
                    }
                }
            } else if self.cannon_cooldown <= 0.0 {
                if let Some(slot) = self.cannon_projectiles.iter_mut().find(|s| !s.active) {
                    if let (Some(spawn), Ok(mut proj_t), Ok(mut v)) = (
                        cannon_spawn,
                        self.world.get::<&mut Transform>(slot.entity),
                        self.world.get::<&mut Velocity>(slot.entity),
                    ) {
                        let mut dir = aim_pos - spawn;
                        if dir.length_squared() < 1.0 {
                            dir = Vec2::new(1.0, 0.0);
                        }
                        dir = dir.normalize();
                        proj_t.pos = spawn;
                        v.vel = dir * cannon_speed;
                        slot.active = true;
                        self.cannon_cooldown = cannon_cadence;
                    }
                }
            }
        }

        for slot in &mut self.arrow_projectiles {
            if let (Ok(mut proj_t), Ok(mut v)) = (
                self.world.get::<&mut Transform>(slot.entity),
                self.world.get::<&mut Velocity>(slot.entity),
            ) {
                match slot.state {
                    ArrowState::Inactive => {
                        if proj_t.pos != hidden_pos {
                            proj_t.pos = hidden_pos;
                        }
                    }
                    ArrowState::Flying => {
                        v.vel.y += arrow_gravity * dt;
                        proj_t.pos += v.vel * dt;
                        if proj_t.pos.y >= ground_impact_y {
                            let rotation = v.vel.y.atan2(v.vel.x);
                            proj_t.pos.y = ground_impact_y;
                            v.vel = Vec2::ZERO;
                            slot.state = ArrowState::Grounded {
                                fade_timer: 0.0,
                                alpha: 1.0,
                                rotation,
                            };
                            continue;
                        }
                        let offscreen = proj_t.pos.x < -projectile_offscreen
                            || proj_t.pos.x > self.viewport_w + projectile_offscreen
                            || proj_t.pos.y > self.viewport_h + projectile_offscreen
                            || proj_t.pos.y < -projectile_offscreen;
                        if offscreen {
                            slot.state = ArrowState::Inactive;
                            proj_t.pos = hidden_pos;
                        }
                    }
                    ArrowState::Grounded {
                        mut fade_timer,
                        mut alpha,
                        rotation,
                    } => {
                        fade_timer += dt;
                        while fade_timer >= ARROW_FADE_INTERVAL {
                            fade_timer -= ARROW_FADE_INTERVAL;
                            alpha = (alpha - ARROW_FADE_STEP).max(0.0);
                        }
                        if alpha <= 0.0 {
                            slot.state = ArrowState::Inactive;
                            proj_t.pos = hidden_pos;
                        } else {
                            slot.state = ArrowState::Grounded {
                                fade_timer,
                                alpha,
                                rotation,
                            };
                        }
                    }
                }
            }
        }

        let mut explosion_spawns: Vec<Vec2> = Vec::new();
        for slot in &mut self.cannon_projectiles {
            if let (Ok(mut proj_t), Ok(mut v)) = (
                self.world.get::<&mut Transform>(slot.entity),
                self.world.get::<&mut Velocity>(slot.entity),
            ) {
                if slot.active {
                    v.vel.y += cannon_gravity * dt;
                    proj_t.pos += v.vel * dt;
                    if proj_t.pos.y >= ground_impact_y {
                        let impact_pos = Vec2::new(proj_t.pos.x, ground_impact_y);
                        slot.active = false;
                        proj_t.pos = hidden_pos;
                        explosion_spawns.push(impact_pos);
                        continue;
                    }
                    let offscreen = proj_t.pos.x < -projectile_offscreen
                        || proj_t.pos.x > self.viewport_w + projectile_offscreen
                        || proj_t.pos.y > self.viewport_h + projectile_offscreen
                        || proj_t.pos.y < -projectile_offscreen;
                    if offscreen {
                        slot.active = false;
                        proj_t.pos = hidden_pos;
                    }
                } else if proj_t.pos != hidden_pos {
                    proj_t.pos = hidden_pos;
                }
            }
        }

        for pos in explosion_spawns {
            self.spawn_explosion(pos);
        }

        for slot in &mut self.explosions {
            if slot.active {
                slot.timer += dt;
                while slot.timer >= EXPLOSION_FRAME_TIME && slot.stage + 1 < EXPLOSION_STAGE_COUNT {
                    slot.timer -= EXPLOSION_FRAME_TIME;
                    slot.stage += 1;
                    if let Ok(mut r) = self.world.get::<&mut Renderable>(slot.entity) {
                        r.sprite_id = SPRITE_EXPLOSION_BASE + slot.stage as u16;
                    }
                }
                if slot.stage + 1 >= EXPLOSION_STAGE_COUNT && slot.timer >= EXPLOSION_FRAME_TIME {
                    slot.active = false;
                    slot.stage = 0;
                    slot.timer = 0.0;
                    if let Ok(mut t) = self.world.get::<&mut Transform>(slot.entity) {
                        t.pos = hidden_pos;
                    }
                } else if let Ok(mut c) = self.world.get::<&mut Collider>(slot.entity) {
                    if let ColliderShape::Circle { .. } = c.shape {
                        let scale = explosion_scale(slot.stage);
                        c.shape = ColliderShape::Circle {
                            radius: projectile_height * 0.5 * scale,
                        };
                    }
                }
            } else if let Ok(mut t) = self.world.get::<&mut Transform>(slot.entity) {
                if t.pos != hidden_pos {
                    t.pos = hidden_pos;
                }
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
                        t.pos = Vec2::new(left_stop, ground_y);
                        v.vel = Vec2::ZERO;
                        self.triguy_state.phase = CharacterPhase::Attacking;
                    }
                }
                CharacterPhase::Attacking => {
                    t.pos = Vec2::new(left_stop, ground_y);
                    v.vel = Vec2::ZERO;
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
                        t.pos = Vec2::new(right_stop, ground_y);
                        v.vel = Vec2::ZERO;
                        self.wedgeguy_state.phase = CharacterPhase::Attacking;
                    }
                }
                CharacterPhase::Attacking => {
                    t.pos = Vec2::new(right_stop, ground_y);
                    v.vel = Vec2::ZERO;
                }
            }
        }
        if let Ok(mut r) = self.world.get::<&mut Renderable>(self.wedgeguy) {
            r.sprite_id = wedgeguy_sprite;
        }

        let mut attackers = 0u32;
        if let CharacterPhase::Attacking = self.triguy_state.phase {
            attackers += 1;
        }
        if let CharacterPhase::Attacking = self.wedgeguy_state.phase {
            attackers += 1;
        }
        if attackers > 0 {
            let damage = WALL_DAMAGE_PER_SECOND * attackers as f32 * dt;
            self.wall_integrity = (self.wall_integrity - damage).max(0.0);
            if self.wall_integrity <= 0.0 {
                self.game_over = true;
            }
        }

        self.resolve_projectile_hits();
    }

    pub fn render_list(&mut self) -> Vec<f32> {
        // Packed: [x, y, rotation, sprite_id, target_height, alpha]. Exposed to JS as a Float32Array.
        // Sprite IDs must match the JS spritePaths order.
        self.update_render_buf();
        self.render_buf.clone()
    }

    pub fn render_list_view(&mut self) -> Float32Array {
        // View into WASM memory; JS must not hold this across resizes that reallocate buffers.
        self.update_render_buf();
        unsafe { Float32Array::view(&self.render_buf) }
    }

    pub fn debug_list(&mut self) -> Vec<f32> {
        // Packed: [kind, x, y, a, b]
        // kind: 0 = circle (a=radius), 1 = aabb (a=half_w, b=half_h)
        self.update_debug_buf();
        self.debug_buf.clone()
    }

    pub fn debug_list_view(&mut self) -> Float32Array {
        // View into WASM memory; JS must not hold this across resizes that reallocate buffers.
        self.update_debug_buf();
        unsafe { Float32Array::view(&self.debug_buf) }
    }
}

impl Game {
    fn sprite_height_for(&self, sprite_id: u16) -> f32 {
        self.sprite_heights_cache
            .get(sprite_id as usize)
            .copied()
            .unwrap_or(0.0)
    }

    fn update_render_buf(&mut self) {
        self.render_buf.clear();
        for entity in [self.castle, self.cannon, self.crossbow] {
            if let (Ok(t), Ok(r)) = (
                self.world.get::<&Transform>(entity),
                self.world.get::<&Renderable>(entity),
            ) {
                self.render_buf.push(t.pos.x);
                self.render_buf.push(t.pos.y);
                self.render_buf.push(0.0);
                self.render_buf.push(r.sprite_id as f32);
                self.render_buf.push(self.sprite_height_for(r.sprite_id));
                self.render_buf.push(1.0);
            }
        }
        for slot in &self.arrow_projectiles {
            if let (Ok(t), Ok(r)) = (
                self.world.get::<&Transform>(slot.entity),
                self.world.get::<&Renderable>(slot.entity),
            ) {
                let (rotation, alpha) = match slot.state {
                    ArrowState::Inactive => (0.0, 0.0),
                    ArrowState::Flying => (
                        self.world
                            .get::<&Velocity>(slot.entity)
                            .map(|v| v.vel.y.atan2(v.vel.x))
                            .unwrap_or(0.0),
                        1.0,
                    ),
                    ArrowState::Grounded { rotation, alpha, .. } => (rotation, alpha),
                };
                self.render_buf.push(t.pos.x);
                self.render_buf.push(t.pos.y);
                self.render_buf.push(rotation);
                self.render_buf.push(r.sprite_id as f32);
                self.render_buf.push(self.sprite_height_for(r.sprite_id));
                self.render_buf.push(alpha);
            }
        }
        for slot in &self.cannon_projectiles {
            if let (Ok(t), Ok(r)) = (
                self.world.get::<&Transform>(slot.entity),
                self.world.get::<&Renderable>(slot.entity),
            ) {
                let rotation = if slot.active {
                    self.world
                        .get::<&Velocity>(slot.entity)
                        .map(|v| v.vel.y.atan2(v.vel.x))
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                self.render_buf.push(t.pos.x);
                self.render_buf.push(t.pos.y);
                self.render_buf.push(rotation);
                self.render_buf.push(r.sprite_id as f32);
                self.render_buf.push(self.sprite_height_for(r.sprite_id));
                self.render_buf.push(1.0);
            }
        }
        for slot in &self.explosions {
            if let (Ok(t), Ok(r)) = (
                self.world.get::<&Transform>(slot.entity),
                self.world.get::<&Renderable>(slot.entity),
            ) {
                self.render_buf.push(t.pos.x);
                self.render_buf.push(t.pos.y);
                self.render_buf.push(0.0);
                self.render_buf.push(r.sprite_id as f32);
                self.render_buf.push(self.sprite_height_for(r.sprite_id));
                self.render_buf.push(1.0);
            }
        }
        for entity in [self.triguy, self.wedgeguy] {
            if let (Ok(t), Ok(r)) = (
                self.world.get::<&Transform>(entity),
                self.world.get::<&Renderable>(entity),
            ) {
                self.render_buf.push(t.pos.x);
                self.render_buf.push(t.pos.y);
                self.render_buf.push(0.0);
                self.render_buf.push(r.sprite_id as f32);
                self.render_buf.push(self.sprite_height_for(r.sprite_id));
                self.render_buf.push(1.0);
            }
        }
    }

    fn update_debug_buf(&mut self) {
        self.debug_buf.clear();
        for (_, (t, c)) in self.world.query::<(&Transform, &Collider)>().iter() {
            let center = t.pos + c.offset;
            match c.shape {
                ColliderShape::Circle { radius } => {
                    self.debug_buf.push(0.0);
                    self.debug_buf.push(center.x);
                    self.debug_buf.push(center.y);
                    self.debug_buf.push(radius);
                    self.debug_buf.push(0.0);
                }
                ColliderShape::Aabb { half_extents } => {
                    self.debug_buf.push(1.0);
                    self.debug_buf.push(center.x);
                    self.debug_buf.push(center.y);
                    self.debug_buf.push(half_extents.x);
                    self.debug_buf.push(half_extents.y);
                }
            }
        }
    }

    fn spawn_explosion(&mut self, pos: Vec2) {
        if let Some(slot) = self.explosions.iter_mut().find(|s| !s.active) {
            slot.active = true;
            slot.stage = 0;
            slot.timer = 0.0;
            if let Ok(mut t) = self.world.get::<&mut Transform>(slot.entity) {
                t.pos = pos;
            }
            if let Ok(mut r) = self.world.get::<&mut Renderable>(slot.entity) {
                r.sprite_id = SPRITE_EXPLOSION_BASE;
            }
            if let Ok(mut c) = self.world.get::<&mut Collider>(slot.entity) {
                c.shape = ColliderShape::Circle {
                    radius: 10.0 * explosion_scale(0),
                };
            }
        }
    }

    fn resolve_projectile_hits(&mut self) {
        let mut score_add = 0u32;
        let hidden_pos = Vec2::new(-10000.0, -10000.0);
        let mut explosion_spawns: Vec<Vec2> = Vec::new();

        let arrow_height = self.sprite_height_for(7);
        let arrow_capsule_half = arrow_height * ARROW_CAPSULE_HALF_RATIO;
        let arrow_capsule_radius = arrow_height * ARROW_CAPSULE_RADIUS_RATIO;
        for slot in &mut self.arrow_projectiles {
            if !matches!(slot.state, ArrowState::Flying) {
                continue;
            }
            let proj_data = if let (Ok(t), Ok(c), Ok(v)) = (
                self.world.get::<&Transform>(slot.entity),
                self.world.get::<&Collider>(slot.entity),
                self.world.get::<&Velocity>(slot.entity),
            ) {
                let rotation = if v.vel.length_squared() > 0.0001 {
                    v.vel.y.atan2(v.vel.x)
                } else {
                    0.0
                };
                let dir = Vec2::new(rotation.cos(), rotation.sin());
                Some((t.pos + c.offset, dir))
            } else {
                None
            };
            if let Some((proj_pos, proj_dir)) = proj_data {
                let mut hit = false;
                if try_hit_character_capsule(
                    &mut self.world,
                    self.triguy,
                    proj_pos,
                    proj_dir,
                    arrow_capsule_half,
                    arrow_capsule_radius,
                    &mut self.triguy_state,
                ) {
                    hit = true;
                    score_add = score_add.saturating_add(1);
                }
                if try_hit_character_capsule(
                    &mut self.world,
                    self.wedgeguy,
                    proj_pos,
                    proj_dir,
                    arrow_capsule_half,
                    arrow_capsule_radius,
                    &mut self.wedgeguy_state,
                ) {
                    hit = true;
                    score_add = score_add.saturating_add(1);
                }
                if hit {
                    slot.state = ArrowState::Inactive;
                    if let Ok(mut t) = self.world.get::<&mut Transform>(slot.entity) {
                        t.pos = hidden_pos;
                    }
                }
            }
        }

        for slot in &mut self.cannon_projectiles {
            if !slot.active {
                continue;
            }
            let proj_data = if let (Ok(t), Ok(c)) = (
                self.world.get::<&Transform>(slot.entity),
                self.world.get::<&Collider>(slot.entity),
            ) {
                if let ColliderShape::Circle { radius } = c.shape {
                    Some((t.pos + c.offset, radius))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some((proj_pos, proj_r)) = proj_data {
                let mut hit = false;
                if try_hit_character(
                    &mut self.world,
                    self.triguy,
                    proj_pos,
                    proj_r,
                    &mut self.triguy_state,
                ) {
                    hit = true;
                    score_add = score_add.saturating_add(1);
                }
                if try_hit_character(
                    &mut self.world,
                    self.wedgeguy,
                    proj_pos,
                    proj_r,
                    &mut self.wedgeguy_state,
                ) {
                    hit = true;
                    score_add = score_add.saturating_add(1);
                }
                if hit {
                    slot.active = false;
                    if let Ok(mut t) = self.world.get::<&mut Transform>(slot.entity) {
                        t.pos = hidden_pos;
                    }
                    explosion_spawns.push(proj_pos);
                }
            }
        }

        for pos in explosion_spawns {
            self.spawn_explosion(pos);
        }

        for slot in &self.explosions {
            if !slot.active {
                continue;
            }
            let explosion_data = if let (Ok(t), Ok(c)) = (
                self.world.get::<&Transform>(slot.entity),
                self.world.get::<&Collider>(slot.entity),
            ) {
                if let ColliderShape::Circle { radius } = c.shape {
                    Some((t.pos + c.offset, radius))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some((explosion_pos, explosion_r)) = explosion_data {
                if try_hit_character(
                    &mut self.world,
                    self.triguy,
                    explosion_pos,
                    explosion_r,
                    &mut self.triguy_state,
                ) {
                    score_add = score_add.saturating_add(1);
                }
                if try_hit_character(
                    &mut self.world,
                    self.wedgeguy,
                    explosion_pos,
                    explosion_r,
                    &mut self.wedgeguy_state,
                ) {
                    score_add = score_add.saturating_add(1);
                }
            }
        }

        if score_add > 0 {
            self.score = self.score.saturating_add(score_add);
        }
    }
}

fn explosion_scale(stage: u8) -> f32 {
    let stage = stage.min(EXPLOSION_STAGE_COUNT.saturating_sub(1));
    if stage < EXPLOSION_GROW_STAGES {
        1.25_f32.powi(stage as i32)
    } else {
        let max_scale = 1.25_f32.powi((EXPLOSION_GROW_STAGES - 1) as i32);
        let shrink_stage = stage - (EXPLOSION_GROW_STAGES - 1);
        max_scale * 0.5_f32.powi(shrink_stage as i32)
    }
}

fn compute_sprite_heights(viewport_w: f32, viewport_h: f32) -> Vec<f32> {
    let total = BASE_SPRITE_COUNT + EXPLOSION_STAGE_COUNT as usize;
    if viewport_w <= 0.0 || viewport_h <= 0.0 {
        return vec![0.0; total];
    }

    let base = viewport_w.min(viewport_h);
    let castle_height = base * 0.25;
    let weapon_height = castle_height * 0.22;
    let character_height = castle_height * 0.35;
    let projectile_height = weapon_height * 0.35;

    let mut out = vec![character_height; total];
    if out.len() > 4 {
        out[4] = castle_height;
    }
    if out.len() > 6 {
        out[5] = weapon_height;
        out[6] = weapon_height;
    }
    if out.len() > 8 {
        out[7] = projectile_height;
        out[8] = projectile_height;
    }
    for stage in 0..EXPLOSION_STAGE_COUNT {
        let idx = SPRITE_EXPLOSION_BASE as usize + stage as usize;
        if let Some(slot) = out.get_mut(idx) {
            *slot = projectile_height * explosion_scale(stage);
        }
    }
    out
}

fn circle_hit(a_pos: Vec2, a_r: f32, b_pos: Vec2, b_r: f32) -> bool {
    let delta = a_pos - b_pos;
    let radius = a_r + b_r;
    delta.length_squared() <= radius * radius
}

fn point_segment_distance_sq(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ab_len_sq = ab.length_squared();
    if ab_len_sq <= 0.0001 {
        return (p - a).length_squared();
    }
    let t = ((p - a).dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let proj = a + ab * t;
    (p - proj).length_squared()
}

fn capsule_hit(
    circle_center: Vec2,
    circle_radius: f32,
    capsule_center: Vec2,
    capsule_dir: Vec2,
    capsule_half: f32,
    capsule_radius: f32,
) -> bool {
    let dir = if capsule_dir.length_squared() > 0.0001 {
        capsule_dir.normalize()
    } else {
        Vec2::new(1.0, 0.0)
    };
    let a = capsule_center - dir * capsule_half;
    let b = capsule_center + dir * capsule_half;
    let dist_sq = point_segment_distance_sq(circle_center, a, b);
    let radius = circle_radius + capsule_radius;
    dist_sq <= radius * radius
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

fn try_hit_character_capsule(
    world: &mut World,
    entity: Entity,
    capsule_center: Vec2,
    capsule_dir: Vec2,
    capsule_half: f32,
    capsule_radius: f32,
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
    if capsule_hit(
        pos,
        radius,
        capsule_center,
        capsule_dir,
        capsule_half,
        capsule_radius,
    ) {
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

    fn rotation_at(list: &[f32], entity_index: usize) -> f32 {
        let base = entity_index * 6;
        list[base + 2]
    }

    fn alpha_at(list: &[f32], entity_index: usize) -> f32 {
        let base = entity_index * 6;
        list[base + 5]
    }

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

    #[test]
    fn render_list_stride_is_6() {
        let mut game = Game::new();
        game.set_viewport(800.0, 600.0);
        let input = InputState::new();
        game.tick(1.0 / 60.0, &input);
        let list = game.render_list();
        let expected_entities = 5 + ARROW_POOL_SIZE + CANNONBALL_POOL_SIZE + EXPLOSION_POOL_SIZE;
        assert_eq!(list.len(), expected_entities * 6);
        assert_eq!(list.len() % 6, 0);
    }

    #[test]
    fn inactive_projectile_rotation_zero() {
        let mut game = Game::new();
        game.set_viewport(800.0, 600.0);
        let input = InputState::new();
        game.tick(1.0 / 60.0, &input);
        let list = game.render_list();
        let arrow_rot = rotation_at(&list, 3);
        let cannon_rot = rotation_at(&list, 3 + ARROW_POOL_SIZE);
        assert!(arrow_rot.abs() < 0.0001);
        assert!(cannon_rot.abs() < 0.0001);
    }

    #[test]
    fn projectile_rotation_changes_when_falling() {
        let mut game = Game::new();
        game.set_viewport(800.0, 600.0);
        let mut input = InputState::new();
        input.pointer_x = 0.0;
        input.pointer_y = 0.0;
        input.pointer_down = true;
        game.tick(1.0 / 60.0, &input);
        let list1 = game.render_list();
        let rot1 = rotation_at(&list1, 3);
        assert!(rot1.abs() > 0.01);

        input.pointer_down = false;
        game.tick(1.0 / 60.0, &input);
        let list2 = game.render_list();
        let rot2 = rotation_at(&list2, 3);
        assert!((rot2 - rot1).abs() > 0.0001);
    }

    #[test]
    fn grounded_arrow_fades_out() {
        let mut game = Game::new();
        game.set_viewport(800.0, 600.0);
        let mut input = InputState::new();
        input.pointer_x = 0.0;
        input.pointer_y = 1000.0;
        input.pointer_down = true;
        game.tick(1.0 / 60.0, &input);
        input.pointer_down = false;

        let mut grounded = false;
        for _ in 0..180 {
            game.tick(1.0 / 60.0, &input);
            if matches!(game.arrow_projectiles[0].state, ArrowState::Grounded { .. }) {
                grounded = true;
                break;
            }
        }
        assert!(grounded);

        let list_before = game.render_list();
        let alpha_before = alpha_at(&list_before, 3);
        game.tick(ARROW_FADE_INTERVAL, &input);
        let list_after = game.render_list();
        let alpha_after = alpha_at(&list_after, 3);

        assert!(alpha_after < alpha_before);
        assert!((alpha_after - (alpha_before - ARROW_FADE_STEP)).abs() < 0.05);
    }

    #[test]
    fn wall_integrity_degrades_and_triggers_game_over() {
        let mut game = Game::new();
        game.set_viewport(800.0, 600.0);
        let input = InputState::new();

        let dt = 1.0 / 30.0;
        let mut elapsed = 0.0;
        while elapsed < 8.0 {
            game.tick(dt, &input);
            elapsed += dt;
        }
        let mid_integrity = game.wall_integrity();
        assert!(mid_integrity < 1.0);
        assert!(mid_integrity > 0.0);

        while elapsed < 30.0 {
            game.tick(dt, &input);
            elapsed += dt;
        }
        assert!(game.game_over());
        assert!(game.wall_integrity() <= 0.001);
    }
}
