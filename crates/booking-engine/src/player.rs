use crate::BookingEngine;
use book_world::ChunkManager;

const PLAYER_RADIUS: f32 = 0.30;
const PLAYER_HEIGHT: f32 = 1.80;
const EYE_HEIGHT: f32 = 1.62;
const COLLISION_EPSILON: f32 = 0.0001;

/// Input sampled by the client once per simulation tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerInput {
    /// Forward/backward axis in the range -1..=1.
    pub forward: f32,
    /// Left/right axis in the range -1..=1.
    pub strafe: f32,
    pub jump: bool,
    pub sprint: bool,
}

/// First-person player simulation with gravity and axis-separated voxel collision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerController {
    position: [f32; 3],
    velocity: [f32; 3],
    yaw: f32,
    pitch: f32,
    on_ground: bool,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            position: [0.5, 40.0, 0.5],
            velocity: [0.0; 3],
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
            on_ground: false,
        }
    }
}

impl PlayerController {
    /// Finds the highest solid block at the requested X/Z and spawns above it.
    pub fn spawn(engine: &BookingEngine, world_x: i32, world_z: i32) -> Option<Self> {
        let registry = engine.block_registry();
        for world_y in (-32..=64).rev() {
            let block = engine.world().get_block(world_x, world_y, world_z)?;
            let solid = registry.get(block).map(|definition| definition.solid).unwrap_or(false);
            if solid {
                return Some(Self {
                    position: [world_x as f32 + 0.5, world_y as f32 + 1.001, world_z as f32 + 0.5],
                    ..Self::default()
                });
            }
        }
        None
    }

    pub fn position(&self) -> [f32; 3] { self.position }
    pub fn velocity(&self) -> [f32; 3] { self.velocity }
    pub fn yaw(&self) -> f32 { self.yaw }
    pub fn pitch(&self) -> f32 { self.pitch }
    pub fn on_ground(&self) -> bool { self.on_ground }

    pub fn camera_position(&self) -> [f32; 3] {
        [self.position[0], self.position[1] + EYE_HEIGHT, self.position[2]]
    }

    pub fn look(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.yaw += yaw_delta;
        self.pitch = (self.pitch + pitch_delta).clamp(-1.5533, 1.5533);
    }

    /// Advances player movement. Delta time is clamped to avoid large lifecycle
    /// or frame-stall steps from tunnelling through the terrain.
    pub fn update(&mut self, engine: &BookingEngine, input: PlayerInput, delta_seconds: f32) {
        let dt = delta_seconds.clamp(0.0, 0.05);
        if dt == 0.0 { return; }

        let mut forward = input.forward.clamp(-1.0, 1.0);
        let mut strafe = input.strafe.clamp(-1.0, 1.0);
        let magnitude = (forward * forward + strafe * strafe).sqrt();
        if magnitude > 1.0 {
            forward /= magnitude;
            strafe /= magnitude;
        }

        let speed = if input.sprint { 6.0 } else { 4.3 };
        let forward_x = self.yaw.cos();
        let forward_z = self.yaw.sin();
        let right_x = -self.yaw.sin();
        let right_z = self.yaw.cos();
        self.velocity[0] = (forward_x * forward + right_x * strafe) * speed;
        self.velocity[2] = (forward_z * forward + right_z * strafe) * speed;

        if input.jump && self.on_ground {
            self.velocity[1] = 8.0;
            self.on_ground = false;
        }
        self.velocity[1] = (self.velocity[1] - 24.0 * dt).max(-50.0);

        self.move_axis(engine, 0, self.velocity[0] * dt);
        self.move_axis(engine, 2, self.velocity[2] * dt);
        self.on_ground = false;
        let vertical_step = self.velocity[1] * dt;
        let previous_vertical_velocity = self.velocity[1];
        self.move_axis(engine, 1, vertical_step);
        if (self.velocity[1] == 0.0) && previous_vertical_velocity < 0.0 {
            self.on_ground = true;
        }
    }

    fn move_axis(&mut self, engine: &BookingEngine, axis: usize, amount: f32) {
        if amount == 0.0 { return; }
        self.position[axis] += amount;
        if self.collides(engine) {
            self.position[axis] -= amount;
            self.velocity[axis] = 0.0;
        }
    }

    fn collides(&self, engine: &BookingEngine) -> bool {
        let min_x = (self.position[0] - PLAYER_RADIUS).floor() as i32;
        let max_x = (self.position[0] + PLAYER_RADIUS - COLLISION_EPSILON).floor() as i32;
        let min_y = self.position[1].floor() as i32;
        let max_y = (self.position[1] + PLAYER_HEIGHT - COLLISION_EPSILON).floor() as i32;
        let min_z = (self.position[2] - PLAYER_RADIUS).floor() as i32;
        let max_z = (self.position[2] + PLAYER_RADIUS - COLLISION_EPSILON).floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    match engine.world().get_block(x, y, z) {
                        Some(block) => {
                            if engine.block_registry().get(block)
                                .map(|definition| definition.solid)
                                .unwrap_or(false)
                            {
                                return true;
                            }
                        }
                        None => {
                            let (chunk, _, _) = ChunkManager::locate(x, z);
                            if !engine.world().is_loaded(chunk) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_spawns_above_generated_surface() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(1);
        let player = PlayerController::spawn(&engine, 0, 0).expect("spawn surface");
        assert!(player.position()[1] > 20.0);
        assert!(player.camera_position()[1] > player.position()[1]);
    }

    #[test]
    fn player_gravity_settles_on_the_ground() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(1);
        let mut player = PlayerController::spawn(&engine, 0, 0).expect("spawn surface");
        for _ in 0..60 {
            player.update(&engine, PlayerInput::default(), 1.0 / 60.0);
        }
        assert!(player.on_ground());
        assert!(player.velocity()[1].abs() < f32::EPSILON);
    }

    #[test]
    fn player_moves_forward_without_free_flight() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(1);
        let mut player = PlayerController::spawn(&engine, 0, 0).expect("spawn surface");
        let start = player.position();
        for _ in 0..30 {
            player.update(&engine, PlayerInput { forward: 1.0, ..Default::default() }, 1.0 / 60.0);
        }
        assert_ne!(player.position()[0], start[0]);
        assert_eq!(player.position()[1], start[1]);
    }
}
