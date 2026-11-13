#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClearColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl ClearColor {
    pub const SKY: Self = Self { red: 0.42, green: 0.68, blue: 0.92, alpha: 1.0 };
    pub const BLACK: Self = Self { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 };
}

/// Camera state shared between the client input layer and renderer backends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraState {
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub vertical_fov_radians: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: [39.0, 19.0, 43.0],
            yaw: -2.36,
            pitch: -0.45,
            vertical_fov_radians: std::f32::consts::FRAC_PI_3,
            near_plane: 0.1,
            far_plane: 256.0,
        }
    }
}

impl CameraState {
    pub fn forward(&self) -> [f32; 3] {
        let horizontal = self.pitch.cos();
        [self.yaw.cos() * horizontal, self.pitch.sin(), self.yaw.sin() * horizontal]
    }

    pub fn move_forward(&mut self, distance: f32) {
        let forward = self.forward();
        self.position[0] += forward[0] * distance;
        self.position[1] += forward[1] * distance;
        self.position[2] += forward[2] * distance;
    }

    pub fn move_right(&mut self, distance: f32) {
        let right = [-self.yaw.sin(), 0.0, self.yaw.cos()];
        self.position[0] += right[0] * distance;
        self.position[2] += right[2] * distance;
    }

    pub fn move_vertical(&mut self, distance: f32) {
        self.position[1] += distance;
    }

    pub fn look(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.yaw += yaw_delta;
        self.pitch = (self.pitch + pitch_delta).clamp(-1.5533, 1.5533);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderFrame {
    pub clear_color: ClearColor,
    pub camera: CameraState,
}

impl Default for RenderFrame {
    fn default() -> Self {
        Self {
            clear_color: ClearColor::SKY,
            camera: CameraState::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphicsApi { OpenGl, OpenGlEs, Vulkan }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RendererInfo {
    pub name: String,
    pub api: GraphicsApi,
    pub api_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RendererError { pub message: String }

impl RendererError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl std::fmt::Display for RendererError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RendererError {}

pub trait RendererBackend {
    fn info(&self) -> RendererInfo;
    fn render(&mut self, frame: &RenderFrame) -> Result<(), RendererError>;
}

#[cfg(test)]
mod camera_tests {
    use super::*;

    #[test]
    fn camera_movement_changes_world_position() {
        let mut camera = CameraState::default();
        let start = camera.position;
        camera.move_forward(1.0);
        assert_ne!(camera.position, start);
    }

    #[test]
    fn camera_pitch_is_clamped_to_avoid_flip() {
        let mut camera = CameraState::default();
        camera.look(0.0, 100.0);
        assert!(camera.pitch < std::f32::consts::FRAC_PI_2);
    }
}

