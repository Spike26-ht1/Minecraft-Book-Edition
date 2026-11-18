//! Authoritative single-player game session and fixed-step player simulation.
//!
//! Rendering reads a snapshot from this crate. The simulation never calls GPU,
//! Android or window APIs.

use glam::Vec3;
use book_world::{BlockId, World};

const TICKS_PER_SECOND: f64 = 20.0;
const FIXED_STEP_SECONDS: f64 = 1.0 / TICKS_PER_SECOND;
const MAX_FRAME_DELTA_SECONDS: f64 = 0.25;
const MAX_STEPS_PER_FRAME: usize = 5;
const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_EYE_HEIGHT: f32 = 1.62;
const PLAYER_SPEED: f32 = 4.3;
const GROUND_ACCELERATION: f32 = 24.0;
const AIR_ACCELERATION: f32 = 5.0;
const GRAVITY: f32 = 24.0;
const TERMINAL_VELOCITY: f32 = 78.4;
const JUMP_VELOCITY: f32 = 8.4;
const COLLISION_EPSILON: f32 = 0.0001;

/// Input collected by the platform layer. Values are normalized before use.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlayerInput {
    pub forward: f32,
    pub strafe: f32,
}

/// Immutable data required to render one game frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderState {
    pub player_position: [f32; 3],
    pub eye_position: [f32; 3],
    pub yaw_radians: f32,
    pub pitch_radians: f32,
    pub grounded: bool,
    pub world_seed: u64,
    pub generator_version: u32,
    pub loaded_columns: usize,
    pub game_tick: u64,
}

/// The player state is owned by the simulation, not by the renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw_radians: f32,
    pub pitch_radians: f32,
    pub grounded: bool,
}

/// Single-player session. World mutation and player physics advance only on
/// fixed simulation ticks; rendering may run at any display frame rate.
pub struct GameSession {
    world: World,
    player: PlayerState,
    input: PlayerInput,
    pending_jump: bool,
    accumulator_seconds: f64,
    game_tick: u64,
}

impl GameSession {
    /// Starts a new world session around the spawn point.
    pub fn new(seed: u64) -> Self {
        let world = World::generate(seed, 2, 4);
        let spawn_y = world.surface_height(0, 0) as f32 + 1.001;

        Self {
            world,
            player: PlayerState {
                position: Vec3::new(0.5, spawn_y, 0.5),
                velocity: Vec3::ZERO,
                yaw_radians: 0.0,
                pitch_radians: -0.12,
                grounded: false,
            },
            input: PlayerInput::default(),
            pending_jump: false,
            accumulator_seconds: 0.0,
            game_tick: 0,
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub const fn player(&self) -> PlayerState {
        self.player
    }

    pub const fn game_tick(&self) -> u64 {
        self.game_tick
    }

    pub fn set_movement(&mut self, forward: f32, strafe: f32) {
        self.input.forward = finite_clamp(forward, -1.0, 1.0);
        self.input.strafe = finite_clamp(strafe, -1.0, 1.0);
    }

    pub fn rotate_view(&mut self, yaw_delta: f32, pitch_delta: f32) {
        if !yaw_delta.is_finite() || !pitch_delta.is_finite() {
            return;
        }
        self.player.yaw_radians += yaw_delta;
        self.player.pitch_radians =
            (self.player.pitch_radians + pitch_delta).clamp(-1.52, 1.52);
    }

    /// Queues one jump. Holding the Android button does not cause repeated
    /// jumps; another press is required after the player lands.
    pub fn request_jump(&mut self) {
        self.pending_jump = true;
    }

    /// Advances the simulation using a fixed 20 Hz clock.
    ///
    /// Large frame stalls are bounded to avoid a spiral of death on mobile.
    /// The returned value is the number of simulation ticks executed.
    pub fn advance_frame(&mut self, frame_delta_seconds: f32) -> usize {
        if !frame_delta_seconds.is_finite() || frame_delta_seconds <= 0.0 {
            return 0;
        }

        self.accumulator_seconds +=
            (frame_delta_seconds as f64).min(MAX_FRAME_DELTA_SECONDS);
        let mut steps = 0;

        while self.accumulator_seconds >= FIXED_STEP_SECONDS
            && steps < MAX_STEPS_PER_FRAME
        {
            self.tick(FIXED_STEP_SECONDS as f32);
            self.accumulator_seconds -= FIXED_STEP_SECONDS;
            steps += 1;
        }

        if steps == MAX_STEPS_PER_FRAME
            && self.accumulator_seconds >= FIXED_STEP_SECONDS
        {
            self.accumulator_seconds = self.accumulator_seconds.min(FIXED_STEP_SECONDS);
        }

        steps
    }

    pub fn render_state(&self) -> RenderState {
        let eye_position = self.player.position + Vec3::Y * PLAYER_EYE_HEIGHT;
        RenderState {
            player_position: self.player.position.to_array(),
            eye_position: eye_position.to_array(),
            yaw_radians: self.player.yaw_radians,
            pitch_radians: self.player.pitch_radians,
            grounded: self.player.grounded,
            world_seed: self.world.seed(),
            generator_version: self.world.generator_version(),
            loaded_columns: self.world.loaded_column_count(),
            game_tick: self.game_tick,
        }
    }

    fn tick(&mut self, delta_seconds: f32) {
        self.game_tick = self.game_tick.wrapping_add(1);
        self.tick_player(delta_seconds);
    }

    fn tick_player(&mut self, delta_seconds: f32) {
        let yaw = self.player.yaw_radians;
        let forward = Vec3::new(yaw.cos(), 0.0, yaw.sin());
        let right = forward.cross(Vec3::Y).normalize_or_zero();
        let wish = (forward * self.input.forward + right * self.input.strafe)
            .normalize_or_zero()
            * PLAYER_SPEED;

        let acceleration = if self.player.grounded {
            GROUND_ACCELERATION
        } else {
            AIR_ACCELERATION
        };
        let blend = (acceleration * delta_seconds).clamp(0.0, 1.0);
        self.player.velocity.x += (wish.x - self.player.velocity.x) * blend;
        self.player.velocity.z += (wish.z - self.player.velocity.z) * blend;

        if self.pending_jump && self.player.grounded {
            self.player.velocity.y = JUMP_VELOCITY;
            self.player.grounded = false;
        }
        self.pending_jump = false;

        self.player.velocity.y =
            (self.player.velocity.y - GRAVITY * delta_seconds).max(-TERMINAL_VELOCITY);

        let delta_x = self.player.velocity.x * delta_seconds;
        let delta_z = self.player.velocity.z * delta_seconds;
        let delta_y = self.player.velocity.y * delta_seconds;

        self.move_axis(0, delta_x);
        self.move_axis(2, delta_z);
        let hit_vertical = self.move_axis(1, delta_y);

        if hit_vertical {
            if delta_y < 0.0 {
                self.player.grounded = true;
            }
            self.player.velocity.y = 0.0;
        } else {
            self.player.grounded = false;
        }

        if self.player.grounded && self.player.velocity.y < 0.0 {
            self.player.velocity.y = 0.0;
        }
    }

    /// Resolves one axis independently. Binary search prevents fast downward
    /// movement from tunnelling through a one-block floor.
    fn move_axis(&mut self, axis: usize, displacement: f32) -> bool {
        if displacement.abs() < f32::EPSILON {
            return false;
        }

        let mut candidate = self.player.position;
        let candidate_position = component(candidate, axis) + displacement;
        set_component(&mut candidate, axis, candidate_position);

        if !self.collides_at(candidate) {
            self.player.position = candidate;
            return false;
        }

        let mut safe_fraction = 0.0_f32;
        let mut blocked_fraction = 1.0_f32;
        for _ in 0..16 {
            let fraction = (safe_fraction + blocked_fraction) * 0.5;
            let mut probe = self.player.position;
            set_component(
                &mut probe,
                axis,
                component(self.player.position, axis) + displacement * fraction,
            );
            if self.collides_at(probe) {
                blocked_fraction = fraction;
            } else {
                safe_fraction = fraction;
            }
        }

        let resolved_position =
            component(self.player.position, axis) + displacement * safe_fraction;
        set_component(&mut self.player.position, axis, resolved_position);
        true
    }

    fn collides_at(&self, feet: Vec3) -> bool {
        let half_width = PLAYER_WIDTH * 0.5;
        let min_x = (feet.x - half_width + COLLISION_EPSILON).floor() as i32;
        let max_x = (feet.x + half_width - COLLISION_EPSILON).floor() as i32;
        let min_y = (feet.y + COLLISION_EPSILON).floor() as i32;
        let max_y = (feet.y + PLAYER_HEIGHT - COLLISION_EPSILON).floor() as i32;
        let min_z = (feet.z - half_width + COLLISION_EPSILON).floor() as i32;
        let max_z = (feet.z + half_width - COLLISION_EPSILON).floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if !self.world.block_at(x, y, z).is_air() {
                        return true;
                    }
                }
            }
        }
        false
    }
}

fn finite_clamp(value: f32, minimum: f32, maximum: f32) -> f32 {
    if value.is_finite() {
        value.clamp(minimum, maximum)
    } else {
        0.0
    }
}

fn component(vector: Vec3, axis: usize) -> f32 {
    match axis {
        0 => vector.x,
        1 => vector.y,
        _ => vector.z,
    }
}

fn set_component(vector: &mut Vec3, axis: usize, value: f32) {
    match axis {
        0 => vector.x = value,
        1 => vector.y = value,
        _ => vector.z = value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn advance(session: &mut GameSession, ticks: usize) {
        for _ in 0..ticks {
            assert_eq!(session.advance_frame(0.05), 1);
        }
    }

    #[test]
    fn session_starts_with_a_loaded_world_and_player_snapshot() {
        let session = GameSession::new(20261116);
        let state = session.render_state();

        assert_eq!(state.world_seed, 20261116);
        assert_eq!(state.loaded_columns, 25);
        assert_eq!(state.game_tick, 0);
        assert!(state.eye_position[1] > state.player_position[1]);
    }

    #[test]
    fn simulation_runs_at_twenty_ticks_per_second_independent_of_render_rate() {
        let mut session = GameSession::new(42);

        assert_eq!(session.advance_frame(1.0 / 60.0), 0);
        assert_eq!(session.advance_frame(1.0 / 60.0), 0);
        assert_eq!(session.advance_frame(1.0 / 60.0), 1);
        assert_eq!(session.game_tick(), 1);
    }

    #[test]
    fn player_falls_to_the_surface_and_becomes_grounded() {
        let mut session = GameSession::new(42);
        advance(&mut session, 80);

        let player = session.player();
        let surface = session.world().surface_height(0, 0) as f32;

        assert!(player.grounded);
        assert!((player.position.y - (surface + 1.0)).abs() < 0.01);
    }

    #[test]
    fn jump_moves_player_upward_after_ground_contact() {
        let mut session = GameSession::new(42);
        advance(&mut session, 80);
        let start_y = session.player().position.y;

        session.request_jump();
        advance(&mut session, 1);

        assert!(session.player().position.y > start_y);
        assert!(!session.player().grounded);
    }

    #[test]
    fn player_movement_is_simulated_and_not_camera_free_flight() {
        let mut session = GameSession::new(42);
        advance(&mut session, 80);
        let start = session.player().position;

        session.set_movement(1.0, 0.0);
        advance(&mut session, 20);
        session.set_movement(0.0, 0.0);

        let end = session.player().position;
        assert!(end.x > start.x + 1.0);
        assert!((end.y - start.y).abs() < 0.01);
    }

    #[test]
    fn player_cannot_walk_through_a_solid_block() {
        let mut session = GameSession::new(42);
        advance(&mut session, 80);
        let surface = session.world().surface_height(0, 0);
        assert!(session
            .world_mut()
            .set_block(1, surface + 1, 0, BlockId::STONE));

        session.set_movement(1.0, 0.0);
        advance(&mut session, 40);

        assert!(session.player().position.x <= 0.701);
    }

    #[test]
    fn large_frame_stalls_are_bounded() {
        let mut session = GameSession::new(42);

        assert!(session.advance_frame(2.0) <= MAX_STEPS_PER_FRAME);
        assert!(session.game_tick() <= MAX_STEPS_PER_FRAME as u64);
    }
}
