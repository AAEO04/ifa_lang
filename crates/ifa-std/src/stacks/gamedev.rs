//! # Game Development Stack
//!
//! Extensions for game development.
//!
//! Features:
//! - Entity Component System (ECS) architecture
//! - Spatial partitioning (grid-based)
//! - Physics helpers (velocity, collision)
//! - Animation and input handling
//!
//! Uses: macroquad, rapier2d, hecs (when available)

use std::collections::HashMap;

/// 2D Vector with comprehensive operations
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };
    pub const UP: Vec2 = Vec2 { x: 0.0, y: -1.0 };
    pub const DOWN: Vec2 = Vec2 { x: 0.0, y: 1.0 };
    pub const LEFT: Vec2 = Vec2 { x: -1.0, y: 0.0 };
    pub const RIGHT: Vec2 = Vec2 { x: 1.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    pub fn splat(v: f32) -> Self {
        Vec2 { x: v, y: v }
    }

    pub fn add(self, other: Vec2) -> Self {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn sub(self, other: Vec2) -> Self {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn mul(self, other: Vec2) -> Self {
        Vec2 {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }

    pub fn scale(self, s: f32) -> Self {
        Vec2 {
            x: self.x * s,
            y: self.y * s,
        }
    }

    pub fn magnitude(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn magnitude_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn normalize(self) -> Self {
        let m = self.magnitude();
        if m > 0.0 {
            Vec2 {
                x: self.x / m,
                y: self.y / m,
            }
        } else {
            Vec2::ZERO
        }
    }

    pub fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn cross(self, other: Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }

    pub fn distance(self, other: Vec2) -> f32 {
        self.sub(other).magnitude()
    }

    pub fn distance_squared(self, other: Vec2) -> f32 {
        self.sub(other).magnitude_squared()
    }

    pub fn lerp(self, other: Vec2, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Vec2 {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    pub fn rotate(self, radians: f32) -> Self {
        let cos = radians.cos();
        let sin = radians.sin();
        Vec2 {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::add(self, other)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::sub(self, other)
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, s: f32) -> Self {
        self.scale(s)
    }
}

/// Axis-Aligned Bounding Box
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2,
}

impl AABB {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        AABB {
            min: Vec2::new(x, y),
            max: Vec2::new(x + width, y + height),
        }
    }

    pub fn from_center(center: Vec2, half_size: Vec2) -> Self {
        AABB {
            min: center.sub(half_size),
            max: center.add(half_size),
        }
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
    pub fn center(&self) -> Vec2 {
        self.min.add(self.max).scale(0.5)
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    pub fn expand(&self, margin: f32) -> Self {
        AABB {
            min: self.min.sub(Vec2::splat(margin)),
            max: self.max.add(Vec2::splat(margin)),
        }
    }
}

/// Entity ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u64);

impl Entity {
    pub fn new(id: u64) -> Self {
        Entity(id)
    }
}

/// Transform component
#[derive(Debug, Clone, Default)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub fn new(x: f32, y: f32) -> Self {
        Transform {
            position: Vec2::new(x, y),
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

/// Velocity component
#[derive(Debug, Clone, Default)]
pub struct Velocity {
    pub linear: Vec2,
    pub angular: f32,
}

/// Collider component
#[derive(Debug, Clone)]
pub struct Collider {
    pub bounds: AABB,
    pub is_trigger: bool,
}

/// Sprite component
#[derive(Debug, Clone)]
pub struct SpriteComponent {
    pub texture: Option<String>,
    pub width: f32,
    pub height: f32,
    pub origin: Vec2,
    pub color: (u8, u8, u8, u8),
}

impl Default for SpriteComponent {
    fn default() -> Self {
        SpriteComponent {
            texture: None,
            width: 32.0,
            height: 32.0,
            origin: Vec2::new(0.5, 0.5),
            color: (255, 255, 255, 255),
        }
    }
}

/// Sparse Set storage for components
/// Provides cache coherence for iteration and O(1) random access.
pub struct SparseSet<T> {
    pub dense: Vec<T>,
    pub entities: Vec<Entity>,
    pub sparse: HashMap<Entity, usize>,
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Self {
            dense: Vec::new(),
            entities: Vec::new(),
            sparse: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity, value: T) {
        if let Some(&index) = self.sparse.get(&entity) {
            self.dense[index] = value;
        } else {
            self.sparse.insert(entity, self.dense.len());
            self.entities.push(entity);
            self.dense.push(value);
        }
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.sparse.get(&entity).map(|&index| &self.dense[index])
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.sparse.get(&entity).map(|&index| &mut self.dense[index])
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.sparse.contains_key(&entity)
    }

    pub fn remove(&mut self, entity: Entity) {
        if let Some(&index) = self.sparse.get(&entity) {
            let last_index = self.dense.len() - 1;
            let last_entity = self.entities[last_index];

            self.dense.swap_remove(index);
            self.entities.swap_remove(index);

            if index != last_index {
                self.sparse.insert(last_entity, index);
            }
            self.sparse.remove(&entity);
        }
    }

    pub fn clear(&mut self) {
        self.dense.clear();
        self.entities.clear();
        self.sparse.clear();
    }
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple ECS World
pub struct World {
    next_entity: u64,
    entities: Vec<Entity>,

    // Component storage using Sparse Sets
    transforms: SparseSet<Transform>,
    velocities: SparseSet<Velocity>,
    colliders: SparseSet<Collider>,
    sprites: SparseSet<SpriteComponent>,
    tags: SparseSet<Vec<String>>,
}

impl World {
    pub fn new() -> Self {
        World {
            next_entity: 1,
            entities: Vec::new(),
            transforms: SparseSet::new(),
            velocities: SparseSet::new(),
            colliders: SparseSet::new(),
            sprites: SparseSet::new(),
            tags: SparseSet::new(),
        }
    }

    /// Spawn a new entity
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_entity);
        self.next_entity += 1;
        self.entities.push(entity);
        entity
    }

    /// Despawn an entity
    pub fn despawn(&mut self, entity: Entity) {
        self.entities.retain(|e| *e != entity);
        self.transforms.remove(entity);
        self.velocities.remove(entity);
        self.colliders.remove(entity);
        self.sprites.remove(entity);
        self.tags.remove(entity);
    }

    /// Add transform
    pub fn add_transform(&mut self, entity: Entity, transform: Transform) {
        self.transforms.insert(entity, transform);
    }

    /// Add velocity
    pub fn add_velocity(&mut self, entity: Entity, velocity: Velocity) {
        self.velocities.insert(entity, velocity);
    }

    /// Add collider
    pub fn add_collider(&mut self, entity: Entity, collider: Collider) {
        self.colliders.insert(entity, collider);
    }

    /// Add sprite
    pub fn add_sprite(&mut self, entity: Entity, sprite: SpriteComponent) {
        self.sprites.insert(entity, sprite);
    }

    /// Add tag
    pub fn add_tag(&mut self, entity: Entity, tag: &str) {
        if let Some(tags) = self.tags.get_mut(entity) {
            tags.push(tag.to_string());
        } else {
            self.tags.insert(entity, vec![tag.to_string()]);
        }
    }

    /// Get transform
    pub fn transform(&self, entity: Entity) -> Option<&Transform> {
        self.transforms.get(entity)
    }

    /// Get mutable transform
    pub fn transform_mut(&mut self, entity: Entity) -> Option<&mut Transform> {
        self.transforms.get_mut(entity)
    }

    /// Query entities with transform and velocity
    pub fn query_movable(&self) -> Vec<Entity> {
        self.transforms.entities
            .iter()
            .filter(|&e| self.velocities.contains(*e))
            .copied()
            .collect()
    }

    /// Query entities with tag
    pub fn query_tagged(&self, tag: &str) -> Vec<Entity> {
        self.entities
            .iter()
            .filter(|e| {
                self.tags
                    .get(*e)
                    .map(|t| t.contains(&tag.to_string()))
                    .unwrap_or(false)
            })
            .copied()
            .collect()
    }


    /// Update physics using parallel CPU orchestration
    #[cfg(feature = "parallel")]
    pub fn update_physics(&mut self, dt: f32) {
        use crate::infra::cpu::CpuContext;

        // 1. Gather Data (Read Phase)
        let entities = self.query_movable();
        // Pack data into a Vec for Rayon
        let mut updates: Vec<(Entity, Vec2, f32, Vec2, f32)> = Vec::with_capacity(entities.len());
        
        for &entity in &entities {
            if let (Some(transform), Some(velocity)) = (
                self.transforms.get(entity),
                self.velocities.get(entity),
            ) {
                updates.push((
                    entity,
                    transform.position,
                    transform.rotation,
                    velocity.linear,
                    velocity.angular,
                ));
            }
        }

        // 2. Compute in Parallel (Work Phase)
        // Returns list of new (Entity, NewPos, NewRot)
        let results = CpuContext::par_map(&updates, |(_e, pos, rot, lin, ang)| {
            (
                pos.add(lin.scale(dt)),
                rot + ang * dt
            )
        });

        // 3. Apply Updates (Write Phase)
        for (i, &entity) in entities.iter().enumerate() {
            if let Some(transform) = self.transforms.get_mut(entity) {
                let (new_pos, new_rot) = results[i];
                transform.position = new_pos;
                transform.rotation = new_rot;
            }
        }
    }

    /// Fallback sequential update
    #[cfg(not(feature = "parallel"))]
    pub fn update_physics(&mut self, dt: f32) {
        for entity in self.query_movable() {
            if let (Some(transform), Some(velocity)) = (
                self.transforms.get_mut(entity),
                self.velocities.get(entity),
            ) {
                transform.position = transform.position.add(velocity.linear.scale(dt));
                transform.rotation += velocity.angular * dt;
            }
        }
    }

    /// Entity count
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// Spatial hash grid for collision detection
pub struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<Entity>>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        SpatialGrid {
            cell_size,
            cells: HashMap::new(),
        }
    }

    fn hash(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }

    /// Clear the grid
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Insert entity at position
    pub fn insert(&mut self, entity: Entity, bounds: &AABB) {
        let min_cell = self.hash(bounds.min);
        let max_cell = self.hash(bounds.max);

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                self.cells.entry((x, y)).or_default().push(entity);
            }
        }
    }

    /// Query nearby entities
    pub fn query(&self, bounds: &AABB) -> Vec<Entity> {
        let min_cell = self.hash(bounds.min);
        let max_cell = self.hash(bounds.max);

        let mut result: Vec<Entity> = Vec::new();
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                if let Some(entities) = self.cells.get(&(x, y)) {
                    result.extend(entities);
                }
            }
        }
        result.sort_by_key(|e: &Entity| e.0);
        result.dedup();
        result
    }
}

/// Input state
#[derive(Debug, Clone, Default)]
pub struct Input {
    pub keys_down: Vec<String>,
    pub keys_pressed: Vec<String>,
    pub keys_released: Vec<String>,
    pub mouse_pos: Vec2,
    pub mouse_down: [bool; 3],
    pub mouse_pressed: [bool; 3],
}

impl Input {
    pub fn new() -> Self {
        Input::default()
    }

    pub fn is_key_down(&self, key: &str) -> bool {
        self.keys_down.iter().any(|k| k == key)
    }

    pub fn is_key_pressed(&self, key: &str) -> bool {
        self.keys_pressed.iter().any(|k| k == key)
    }

    pub fn clear_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_pressed = [false; 3];
    }
}

/// Timer with repeating support
#[derive(Debug, Clone)]
pub struct GameTimer {
    pub duration: f32,
    pub elapsed: f32,
    pub repeating: bool,
    pub finished: bool,
}

impl GameTimer {
    pub fn once(duration: f32) -> Self {
        GameTimer {
            duration,
            elapsed: 0.0,
            repeating: false,
            finished: false,
        }
    }

    pub fn repeating(duration: f32) -> Self {
        GameTimer {
            duration,
            elapsed: 0.0,
            repeating: true,
            finished: false,
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if self.finished && !self.repeating {
            return false;
        }

        self.elapsed += dt;
        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed -= self.duration;
            } else {
                self.finished = true;
            }
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.finished = false;
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).min(1.0)
    }
}

/// Animation controller
#[derive(Debug, Clone)]
pub struct Animation {
    pub frames: Vec<String>,
    pub frame_duration: f32,
    pub current_frame: usize,
    pub looping: bool,
    timer: GameTimer,
}

impl Animation {
    pub fn new(frames: Vec<String>, fps: f32, looping: bool) -> Self {
        let duration = 1.0 / fps;
        Animation {
            frames,
            frame_duration: duration,
            current_frame: 0,
            looping,
            timer: GameTimer::repeating(duration),
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.timer.tick(dt) {
            self.current_frame += 1;
            if self.current_frame >= self.frames.len() {
                if self.looping {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frames.len() - 1;
                }
            }
        }
    }

    pub fn current(&self) -> &str {
        &self.frames[self.current_frame]
    }

    pub fn is_finished(&self) -> bool {
        !self.looping && self.current_frame >= self.frames.len() - 1
    }
}

/// Game Audio System using rodio
#[cfg(feature = "audio")]
pub struct Audio {
    _stream: Option<rodio::OutputStream>,
    stream_handle: Option<rodio::OutputStreamHandle>,
    music_sink: Option<rodio::Sink>,
}

#[cfg(feature = "audio")]
impl Audio {
    pub fn new() -> Result<Self, String> {
        let (stream, handle) = rodio::OutputStream::try_default()
            .map_err(|e| format!("No audio device: {}", e))?;
        Ok(Audio {
            _stream: Some(stream),
            stream_handle: Some(handle),
            music_sink: None,
        })
    }
    
    /// Play a sound effect (one-shot)
    pub fn play_sound(&self, path: &str, volume: f32) -> Result<(), String> {
        use rodio::{Decoder, Sink};
        use std::fs::File;
        use std::io::BufReader;
        
        let handle = self.stream_handle.as_ref()
            .ok_or("No audio stream")?;
        
        let file = File::open(path).map_err(|e| format!("Cannot open: {}", e))?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| format!("Cannot decode: {}", e))?;
        
        let sink = Sink::try_new(handle)
            .map_err(|e| format!("Cannot create sink: {}", e))?;
        sink.set_volume(volume.clamp(0.0, 1.0));
        sink.append(source);
        sink.detach(); // Let it play to completion
        Ok(())
    }
    
    /// Start looping background music
    pub fn play_music(&mut self, path: &str, looping: bool) -> Result<(), String> {
        use rodio::{Decoder, Sink, Source};
        use std::fs::File;
        use std::io::BufReader;
        
        // Stop existing music
        self.stop_music();
        
        let handle = self.stream_handle.as_ref()
            .ok_or("No audio stream")?;
        
        let file = File::open(path).map_err(|e| format!("Cannot open: {}", e))?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| format!("Cannot decode: {}", e))?;
        
        let sink = Sink::try_new(handle)
            .map_err(|e| format!("Cannot create sink: {}", e))?;
        
        if looping {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }
        
        self.music_sink = Some(sink);
        Ok(())
    }
    
    /// Stop background music
    pub fn stop_music(&mut self) {
        if let Some(sink) = self.music_sink.take() {
            sink.stop();
        }
    }
    
    /// Set music volume
    pub fn set_volume(&self, volume: f32) {
        if let Some(sink) = &self.music_sink {
            sink.set_volume(volume.clamp(0.0, 1.0));
        }
    }
    
    /// Pause music
    pub fn pause_music(&self) {
        if let Some(sink) = &self.music_sink {
            sink.pause();
        }
    }
    
    /// Resume music
    pub fn resume_music(&self) {
        if let Some(sink) = &self.music_sink {
            sink.play();
        }
    }
}

#[cfg(feature = "audio")]
impl Default for Audio {
    fn default() -> Self {
        Audio::new().unwrap_or(Audio {
            _stream: None,
            stream_handle: None,
            music_sink: None,
        })
    }
}

/// Fallback Audio stub (no audio feature)
#[cfg(not(feature = "audio"))]
pub struct Audio;

#[cfg(not(feature = "audio"))]
impl Audio {
    pub fn new() -> Result<Self, String> { Ok(Audio) }
    pub fn play_sound(&self, name: &str, volume: f32) -> Result<(), String> {
        eprintln!("[AUDIO] Play: {} (vol: {}) - audio feature disabled", name, volume);
        Ok(())
    }
    pub fn play_music(&mut self, name: &str, looping: bool) -> Result<(), String> {
        eprintln!("[AUDIO] Music: {} (loop: {}) - audio feature disabled", name, looping);
        Ok(())
    }
    pub fn stop_music(&mut self) {}
    pub fn set_volume(&self, _volume: f32) {}
}

#[cfg(not(feature = "audio"))]
impl Default for Audio {
    fn default() -> Self { Audio }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_ops() {
        let a = Vec2::new(3.0, 4.0);
        assert!((a.magnitude() - 5.0).abs() < 0.001);

        let b = a.normalize();
        assert!((b.magnitude() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_aabb_collision() {
        let a = AABB::new(0.0, 0.0, 10.0, 10.0);
        let b = AABB::new(5.0, 5.0, 10.0, 10.0);
        let c = AABB::new(20.0, 20.0, 10.0, 10.0);

        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_world_spawn() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();

        assert_ne!(e1, e2);
        assert_eq!(world.entity_count(), 2);

        world.despawn(e1);
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_spatial_grid() {
        let mut grid = SpatialGrid::new(50.0);
        let e1 = Entity::new(1);
        let e2 = Entity::new(2);

        grid.insert(e1, &AABB::new(0.0, 0.0, 10.0, 10.0));
        grid.insert(e2, &AABB::new(100.0, 100.0, 10.0, 10.0));

        let near = grid.query(&AABB::new(0.0, 0.0, 20.0, 20.0));
        assert!(near.contains(&e1));
        assert!(!near.contains(&e2));
    }

    #[test]
    fn test_timer() {
        let mut timer = GameTimer::once(1.0);
        assert!(!timer.tick(0.5));
        assert!(timer.tick(0.6));
        assert!(timer.finished);
    }
}
