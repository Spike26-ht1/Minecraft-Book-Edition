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

/// Renderer and frame-pacing options used by the mobile test settings menu.
/// A zero FPS limit means uncapped rendering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderSettings {
    pub fps_limit: u16,
    pub render_distance_chunks: u8,
    pub menu_open: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            fps_limit: 60,
            render_distance_chunks: 4,
            menu_open: false,
        }
    }
}

impl RenderSettings {
    pub const FPS_OPTIONS: [u16; 4] = [30, 60, 90, 0];
    pub const DISTANCE_OPTIONS: [u8; 5] = [2, 4, 6, 8, 12];
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MobileControlsState {
    /// Horizontal movement: left=-1, right=1.
    pub strafe: f32,
    /// Forward movement: forward=1, backward=-1.
    pub forward: f32,
    pub jump_pressed: bool,
    pub sprint_pressed: bool,
    pub break_pressed: bool,
    pub place_pressed: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TouchRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl TouchRect {
    pub fn contains(&self, point: [f32; 2]) -> bool {
        point[0] >= self.x
            && point[0] <= self.x + self.width
            && point[1] >= self.y
            && point[1] <= self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MobileControlsLayout {
    pub joystick_center: [f32; 2],
    pub joystick_radius: f32,
    pub jump: TouchRect,
    pub sprint: TouchRect,
    pub break_block: TouchRect,
    pub place_block: TouchRect,
    pub debug: TouchRect,
    pub settings: TouchRect,
    pub settings_panel: TouchRect,
    pub settings_close: TouchRect,
    pub fps_options: [TouchRect; 4],
    pub distance_options: [TouchRect; 5],
}

impl MobileControlsLayout {
    /// Shared pixel-coordinate layout used by both rendering and touch hit tests.
    pub fn new(width: u32, height: u32) -> Self {
        let width = width.max(1) as f32;
        let height = height.max(1) as f32;
        let button_size = (height * 0.088).clamp(56.0, 76.0).min(width * 0.22);
        let gap = (button_size * 0.14).max(8.0);
        let side_margin = (width * 0.035).clamp(16.0, 32.0);
        let bottom_safe = (height * 0.15).clamp(84.0, 132.0);
        let right_x = (width - side_margin - (button_size * 2.0 + gap)).max(width * 0.40);
        let top_y = (height - bottom_safe - (button_size * 2.0 + gap)).max(height * 0.35);
        let rect = |x: f32, y: f32| TouchRect {
            x, y, width: button_size, height: button_size,
        };
        let joystick_radius = (height * 0.095).clamp(54.0, 82.0).min(width * 0.22);
        let joystick_center = [
            (width * 0.15).clamp(joystick_radius + 12.0, width * 0.34),
            height - bottom_safe - joystick_radius,
        ];
        let top_inset = (height * 0.07).clamp(20.0, 44.0);
        let panel_width = (width - 40.0).min(680.0);
        let panel_height = (height - 72.0).min(500.0);
        let panel_x = (width - panel_width) * 0.5;
        let panel_y = (height - panel_height) * 0.5;
        let panel = TouchRect {
            x: panel_x, y: panel_y, width: panel_width, height: panel_height,
        };
        let menu_gap = 10.0;
        let fps_width = (panel_width - 40.0 - menu_gap * 3.0) / 4.0;
        let distance_width = (panel_width - 40.0 - menu_gap * 4.0) / 5.0;
        let fps_y = panel_y + panel_height * 0.37;
        let distance_y = panel_y + panel_height * 0.69;
        let fps_options = std::array::from_fn(|index| TouchRect {
            x: panel_x + 20.0 + index as f32 * (fps_width + menu_gap),
            y: fps_y,
            width: fps_width,
            height: 58.0,
        });
        let distance_options = std::array::from_fn(|index| TouchRect {
            x: panel_x + 20.0 + index as f32 * (distance_width + menu_gap),
            y: distance_y,
            width: distance_width,
            height: 58.0,
        });

        Self {
            joystick_center,
            joystick_radius,
            jump: rect(right_x, top_y),
            sprint: rect(right_x + button_size + gap, top_y),
            break_block: rect(right_x, top_y + button_size + gap),
            place_block: rect(right_x + button_size + gap, top_y + button_size + gap),
            debug: TouchRect {
                x: width - side_margin - 64.0,
                y: top_inset,
                width: 56.0,
                height: 34.0,
            },
            settings: TouchRect {
                x: width - side_margin - 128.0,
                y: top_inset,
                width: 56.0,
                height: 34.0,
            },
            settings_panel: panel,
            settings_close: TouchRect {
                x: panel_x + panel_width - 82.0,
                y: panel_y + 18.0,
                width: 64.0,
                height: 42.0,
            },
            fps_options,
            distance_options,
        }
    }

    pub fn joystick_contains(&self, point: [f32; 2]) -> bool {
        let dx = point[0] - self.joystick_center[0];
        let dy = point[1] - self.joystick_center[1];
        dx * dx + dy * dy <= self.joystick_radius * self.joystick_radius * 1.35
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderFrame {
    pub clear_color: ClearColor,
    pub camera: CameraState,
    pub mobile_controls: MobileControlsState,
    pub settings: RenderSettings,
}

impl Default for RenderFrame {
    fn default() -> Self {
        Self {
            clear_color: ClearColor::SKY,
            camera: CameraState::default(),
            mobile_controls: MobileControlsState::default(),
            settings: RenderSettings::default(),
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



#[cfg(test)]
mod mobile_controls_tests {
    use super::*;

    #[test]
    fn mobile_controls_fit_common_phone_viewports() {
        for (width, height) in [(360, 800), (393, 873), (720, 1600), (1536, 864)] {
            let layout = MobileControlsLayout::new(width, height);
            let bounds = [
                layout.jump, layout.sprint, layout.break_block,
                layout.place_block, layout.debug, layout.settings,
                layout.settings_panel, layout.settings_close,
            ];
            let mut option_bounds = Vec::from(layout.fps_options);
            option_bounds.extend(layout.distance_options);
            for rect in bounds.into_iter().chain(option_bounds) {
                assert!(rect.x >= 0.0 && rect.y >= 0.0);
                assert!(rect.x + rect.width <= width as f32);
                assert!(rect.y + rect.height <= height as f32);
            }
            assert!(layout.joystick_center[0] - layout.joystick_radius >= 0.0);
            assert!(layout.joystick_center[1] + layout.joystick_radius <= height as f32);
        }
    }

    #[test]
    fn mobile_action_buttons_do_not_overlap_each_other() {
        let layout = MobileControlsLayout::new(393, 873);
        let buttons = [layout.jump, layout.sprint, layout.break_block, layout.place_block];
        for left in 0..buttons.len() {
            for right in (left + 1)..buttons.len() {
                let a = buttons[left];
                let b = buttons[right];
                let overlap_x = a.x < b.x + b.width && a.x + a.width > b.x;
                let overlap_y = a.y < b.y + b.height && a.y + a.height > b.y;
                assert!(!(overlap_x && overlap_y));
            }
        }
    }

    #[test]
    fn settings_option_targets_fit_inside_the_settings_panel() {
        let layout = MobileControlsLayout::new(1536, 864);
        for rect in layout.fps_options.into_iter().chain(layout.distance_options) {
            assert!(rect.x >= layout.settings_panel.x);
            assert!(rect.y >= layout.settings_panel.y);
            assert!(rect.x + rect.width <= layout.settings_panel.x + layout.settings_panel.width);
            assert!(rect.y + rect.height <= layout.settings_panel.y + layout.settings_panel.height);
        }
    }

    #[test]
    fn touch_rect_uses_inclusive_edges() {
        let rect = TouchRect { x: 10.0, y: 20.0, width: 30.0, height: 40.0 };
        assert!(rect.contains([10.0, 20.0]));
        assert!(rect.contains([40.0, 60.0]));
        assert!(!rect.contains([41.0, 60.0]));
    }
}
