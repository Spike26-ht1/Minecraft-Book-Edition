use book_assets::{TextureAtlas, TextureImage};
use booking_engine::BookingEngine;
use book_core::BlockId;
use book_renderer_api::{CameraState, GraphicsApi, MobileControlsLayout, MobileControlsState, RenderSettings, RendererBackend, RendererError, RendererInfo, RenderFrame};
#[cfg(not(target_os = "android"))]
use glium::backend::glutin::SimpleWindowBuilder;
#[cfg(target_os = "android")]
use glutin::config::{Api, ConfigTemplateBuilder};
#[cfg(target_os = "android")]
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
#[cfg(target_os = "android")]
use glutin::display::{GetGlDisplay, GlDisplay};
#[cfg(target_os = "android")]
use glutin::prelude::NotCurrentGlContext;
#[cfg(target_os = "android")]
use glutin::surface::SurfaceAttributesBuilder;
#[cfg(target_os = "android")]
use glutin_winit::DisplayBuilder;
#[cfg(target_os = "android")]
use raw_window_handle::HasWindowHandle;
#[cfg(target_os = "android")]
use std::num::NonZeroU32;
use glium::index::NoIndices;
use glium::texture::{RawImage2d, Texture2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter};
use glium::uniform;
use glium::{implement_vertex, Blend, Depth, DepthTest, DrawParameters, Program, Surface, VertexBuffer};
use glutin::surface::WindowSurface;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[cfg(target_os = "android")]
use std::fs::OpenOptions;
#[cfg(target_os = "android")]
use std::io::Write;
#[cfg(target_os = "android")]
use std::path::PathBuf;
#[cfg(target_os = "android")]
use std::sync::OnceLock;
#[cfg(target_os = "android")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "android")]
static ANDROID_DIAGNOSTIC_LOG_PATHS: OnceLock<[PathBuf; 2]> = OnceLock::new();

/// Configure persistent startup diagnostics for the Android game and launcher logs.
#[cfg(target_os = "android")]
pub fn set_android_diagnostic_log_paths(game_log: PathBuf, launcher_log: PathBuf) {
    let _ = ANDROID_DIAGNOSTIC_LOG_PATHS.set([game_log, launcher_log]);
}

/// Write an Android startup event to Logcat and, once configured, to both app log files.
#[cfg(target_os = "android")]
pub fn record_android_diagnostic(message: &str) {
    log::info!("Android startup: {message}");
    let Some(paths) = ANDROID_DIAGNOSTIC_LOG_PATHS.get() else {
        return;
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    for path in paths {
        match OpenOptions::new().create(true).append(true).open(path) {
            Ok(mut file) => {
                if let Err(error) = writeln!(file, "[{timestamp}] {message}") {
                    log::warn!(
                        "Could not write Android diagnostic log {}: {error}",
                        path.display()
                    );
                }
            }
            Err(error) => log::warn!(
                "Could not open Android diagnostic log {}: {error}",
                path.display()
            ),
        }
    }
}


#[cfg(target_os = "android")]
const VERTEX_SHADER: &str = r#"#version 300 es
precision highp float;
in vec3 position;
in vec2 uv;
in vec3 normal;
in vec3 color;
out vec2 v_uv;
out vec3 v_normal;
out vec3 v_color;
uniform mat4 u_view_projection;
void main() {
    gl_Position = u_view_projection * vec4(position, 1.0);
    v_uv = uv;
    v_normal = normal;
    v_color = color;
}
"#;

#[cfg(target_os = "android")]
const FRAGMENT_SHADER: &str = r#"#version 300 es
precision highp float;
in vec2 v_uv;
in vec3 v_normal;
in vec3 v_color;
uniform sampler2D u_atlas;
out vec4 out_color;
void main() {
    vec3 sun_direction = normalize(vec3(-0.4, 0.8, -0.3));
    float diffuse = max(dot(normalize(v_normal), sun_direction), 0.0);
    float lighting = 0.42 + diffuse * 0.58;
    vec4 texel = texture(u_atlas, v_uv);
    // Keep geometry readable even when a resource atlas is missing, incomplete,
    // or has an unexpected tile orientation. The block tint is the safe fallback.
    vec3 base_color = max(texel.rgb, v_color * 0.72);
    out_color = vec4(base_color * lighting, 1.0);
}
"#;

#[cfg(target_os = "android")]
const DEBUG_VERTEX_SHADER: &str = r#"#version 300 es
precision highp float;
in vec2 position;
in vec4 color;
out vec4 v_color;
uniform vec2 u_screen;
void main() {
    vec2 clip = vec2(position.x / u_screen.x * 2.0 - 1.0,
                     1.0 - position.y / u_screen.y * 2.0);
    gl_Position = vec4(clip, 0.0, 1.0);
    v_color = color;
}
"#;

#[cfg(target_os = "android")]
const DEBUG_FRAGMENT_SHADER: &str = r#"#version 300 es
precision highp float;
in vec4 v_color;
out vec4 out_color;
void main() { out_color = v_color; }
"#;

#[derive(Clone, Copy)]
struct Vertex { position: [f32; 3], uv: [f32; 2], normal: [f32; 3], color: [f32; 3] }
implement_vertex!(Vertex, position, uv, normal, color);

#[derive(Clone, Copy)]
struct DebugVertex { position: [f32; 2], color: [f32; 4] }
implement_vertex!(DebugVertex, position, color);

#[cfg(not(target_os = "android"))]
const VERTEX_SHADER: &str = r#"#version 140
in vec3 position;
in vec2 uv;
in vec3 normal;
in vec3 color;
out vec2 v_uv;
out vec3 v_normal;
out vec3 v_color;
uniform mat4 u_view_projection;
void main() {
    gl_Position = u_view_projection * vec4(position, 1.0);
    v_uv = uv;
    v_normal = normal;
    v_color = color;
}
"#;

#[cfg(not(target_os = "android"))]
const FRAGMENT_SHADER: &str = r#"#version 140
in vec2 v_uv;
in vec3 v_normal;
in vec3 v_color;
uniform sampler2D u_atlas;
out vec4 out_color;
void main() {
    vec3 sun_direction = normalize(vec3(-0.4, 0.8, -0.3));
    float diffuse = max(dot(normalize(v_normal), sun_direction), 0.0);
    float lighting = 0.42 + diffuse * 0.58;
    vec4 texel = texture(u_atlas, v_uv);
    // Keep geometry readable even when a resource atlas is missing, incomplete,
    // or has an unexpected tile orientation. The block tint is the safe fallback.
    vec3 base_color = max(texel.rgb, v_color * 0.72);
    out_color = vec4(base_color * lighting, 1.0);
}
"#;

#[cfg(not(target_os = "android"))]
const DEBUG_VERTEX_SHADER: &str = r#"#version 140
in vec2 position;
in vec4 color;
out vec4 v_color;
uniform vec2 u_screen;
void main() {
    vec2 clip = vec2(position.x / u_screen.x * 2.0 - 1.0,
                     1.0 - position.y / u_screen.y * 2.0);
    gl_Position = vec4(clip, 0.0, 1.0);
    v_color = color;
}
"#;

#[cfg(not(target_os = "android"))]
const DEBUG_FRAGMENT_SHADER: &str = r#"#version 140
in vec4 v_color;
out vec4 out_color;
void main() { out_color = v_color; }
"#;

struct ChunkGpuMesh {
    chunk_x: i32,
    chunk_z: i32,
    vertices: VertexBuffer<Vertex>,
}

pub struct OpenGlRenderer {
    display: glium::backend::glutin::Display<WindowSurface>,
    chunk_meshes: Vec<ChunkGpuMesh>,
    program: Program,
    atlas: Texture2d,
    debug_program: Program,
    last_frame: Instant,
    fps_window_started: Instant,
    frames_in_window: u32,
    fps: u32,
    last_frame_ms: f32,
    debug_visible: bool,
    visible_chunk_count: usize,
    debug_overlay_buffer: VertexBuffer<DebugVertex>,
    debug_overlay_vertices: Vec<DebugVertex>,
    #[cfg(target_os = "android")]
    mobile_controls_buffer: VertexBuffer<DebugVertex>,
    #[cfg(target_os = "android")]
    mobile_controls_vertices: Vec<DebugVertex>,
    #[cfg(target_os = "android")]
    first_frame_logged: bool,
}

impl OpenGlRenderer {
    #[cfg(not(target_os = "android"))]
    pub fn new(
        event_loop: &glium::winit::event_loop::EventLoop<()>,
        title: &str,
        width: u32,
        height: u32,
        engine: &BookingEngine,
    ) -> Result<(glium::winit::window::Window, Self), RendererError> {
        let (window, display) = SimpleWindowBuilder::new()
            .with_title(title)
            .with_inner_size(width, height)
            .build(event_loop);
        Self::from_display(window, display, engine)
    }

    #[cfg(target_os = "android")]
    pub fn new(
        event_loop: &glium::winit::event_loop::ActiveEventLoop,
        title: &str,
        _width: u32,
        _height: u32,
        engine: &BookingEngine,
    ) -> Result<(glium::winit::window::Window, Self), RendererError> {
        let (window, display) = create_android_display(event_loop, title)?;
        Self::from_display(window, display, engine)
    }

    fn from_display(
        window: glium::winit::window::Window,
        display: glium::backend::glutin::Display<WindowSurface>,
        engine: &BookingEngine,
    ) -> Result<(glium::winit::window::Window, Self), RendererError> {
        #[cfg(target_os = "android")]
        record_android_diagnostic("Renderer: building chunk-manager world mesh");
        let chunk_meshes = upload_chunk_meshes(&display, engine)?;
        #[cfg(target_os = "android")]
        record_android_diagnostic("Renderer: uploading prototype texture atlas");
        let atlas_data = prototype_texture_atlas();
        let raw_atlas = RawImage2d::from_raw_rgba_reversed(
            &atlas_data.rgba,
            (atlas_data.width, atlas_data.height),
        );
        let atlas = Texture2d::new(&display, raw_atlas)
            .map_err(|error| RendererError::new(format!("Texture atlas upload: {error}")))?;
        #[cfg(target_os = "android")]
        record_android_diagnostic("Renderer: compiling chunk shader program");
        let program = Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None)
            .map_err(|error| RendererError::new(format!("Chunk shader program: {error}")))?;
        #[cfg(target_os = "android")]
        record_android_diagnostic("Renderer: compiling debug overlay shader program");
        let debug_program = Program::from_source(
            &display, DEBUG_VERTEX_SHADER, DEBUG_FRAGMENT_SHADER, None,
        ).map_err(|error| RendererError::new(format!("Debug overlay shader: {error}")))?;

        let (screen_width, screen_height) = display.get_framebuffer_dimensions();
        let mut debug_overlay_vertices =
            build_debug_overlay_vertices(screen_width, screen_height, 0, 0.0, 0, 0);
        let debug_overlay_buffer = VertexBuffer::dynamic(&display, &debug_overlay_vertices)
            .map_err(|error| RendererError::new(format!("Debug overlay buffer allocation: {error}")))?;

        #[cfg(target_os = "android")]
        let (mobile_controls_buffer, mobile_controls_vertices) = {
            let layout = MobileControlsLayout::new(screen_width, screen_height);
            let mut vertices = build_mobile_controls_vertices(
                layout,
                MobileControlsState::default(),
                RenderSettings::default(),
                false,
            );
            let buffer = VertexBuffer::dynamic(&display, &vertices)
                .map_err(|error| RendererError::new(format!("Mobile controls buffer allocation: {error}")))?;
            (buffer, vertices)
        };

        #[cfg(target_os = "android")]
        record_android_diagnostic("Renderer: GPU resources and persistent HUD buffers are ready");
        let now = Instant::now();
        Ok((window, Self {
            display, chunk_meshes, program, atlas, debug_program,
            last_frame: now, fps_window_started: now,
            frames_in_window: 0, fps: 0, last_frame_ms: 0.0, debug_visible: false,
            visible_chunk_count: 0,
            debug_overlay_buffer,
            debug_overlay_vertices,
            #[cfg(target_os = "android")]
            mobile_controls_buffer,
            #[cfg(target_os = "android")]
            mobile_controls_vertices,
            #[cfg(target_os = "android")]
            first_frame_logged: false,
        }))
    }

    /// Uploads only newly loaded chunk columns and their loaded neighbors.
    /// Existing GPU meshes are retained, avoiding full-world rebuilds whenever
    /// the player crosses a chunk boundary.
    pub fn sync_world(&mut self, engine: &BookingEngine) -> Result<(), RendererError> {
        let loaded: BTreeSet<(i32, i32)> = engine.world()
            .iter()
            .map(|(position, _)| (position.x, position.z))
            .collect();
        let existing: BTreeSet<(i32, i32)> = self.chunk_meshes
            .iter()
            .map(|mesh| (mesh.chunk_x, mesh.chunk_z))
            .collect();
        let missing: BTreeSet<(i32, i32)> =
            loaded.difference(&existing).copied().collect();
        if missing.is_empty() {
            return Ok(());
        }

        let mut affected = missing.clone();
        for &(chunk_x, chunk_z) in &missing {
            for neighbor in [
                (chunk_x - 1, chunk_z),
                (chunk_x + 1, chunk_z),
                (chunk_x, chunk_z - 1),
                (chunk_x, chunk_z + 1),
            ] {
                if loaded.contains(&neighbor) {
                    affected.insert(neighbor);
                }
            }
        }
        self.rebuild_chunk_buffers(engine, &affected)
    }

    /// Rebuilds one edited column and only those cardinal neighbors whose
    /// boundary faces can have changed.
    pub fn sync_chunk(
        &mut self,
        engine: &BookingEngine,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Result<(), RendererError> {
        let mut affected = BTreeSet::from([(chunk_x, chunk_z)]);
        for neighbor in [
            (chunk_x - 1, chunk_z),
            (chunk_x + 1, chunk_z),
            (chunk_x, chunk_z - 1),
            (chunk_x, chunk_z + 1),
        ] {
            let position = book_world::ChunkPos { x: neighbor.0, z: neighbor.1 };
            if engine.world().is_loaded(position) {
                affected.insert(neighbor);
            }
        }
        self.rebuild_chunk_buffers(engine, &affected)
    }

    fn rebuild_chunk_buffers(
        &mut self,
        engine: &BookingEngine,
        positions: &BTreeSet<(i32, i32)>,
    ) -> Result<(), RendererError> {
        let replacement = upload_chunk_meshes_for_chunks(&self.display, engine, positions)?;
        self.chunk_meshes.retain(|mesh| {
            !positions.contains(&(mesh.chunk_x, mesh.chunk_z))
        });
        self.chunk_meshes.extend(replacement);
        self.visible_chunk_count = 0;
        Ok(())
    }

    pub fn toggle_debug(&mut self) { self.debug_visible = !self.debug_visible; }

    fn update_frame_stats(&mut self) {
        let now = Instant::now();
        self.last_frame_ms = now.duration_since(self.last_frame).as_secs_f32() * 1000.0;
        self.last_frame = now;
        self.frames_in_window += 1;
        let elapsed = now.duration_since(self.fps_window_started).as_secs_f32();
        if elapsed >= 1.0 {
            self.fps = (self.frames_in_window as f32 / elapsed).round() as u32;
            self.frames_in_window = 0;
            self.fps_window_started = now;
        }
    }

    fn draw_debug_overlay(&mut self, target: &mut glium::Frame) -> Result<(), RendererError> {
        let vertex_count: usize = self.chunk_meshes.iter().map(|mesh| mesh.vertices.len()).sum();
        let (screen_width, screen_height) = self.display.get_framebuffer_dimensions();
        self.debug_overlay_vertices.clear();
        self.debug_overlay_vertices.extend(build_debug_overlay_vertices(
            screen_width,
            screen_height,
            self.fps,
            self.last_frame_ms,
            self.visible_chunk_count,
            vertex_count,
        ));

        if self.debug_overlay_vertices.len() != self.debug_overlay_buffer.len() {
            return Err(RendererError::new("Debug overlay vertex count changed unexpectedly"));
        }
        self.debug_overlay_buffer.write(&self.debug_overlay_vertices);

        let params = DrawParameters {
            blend: Blend::alpha_blending(),
            ..DrawParameters::default()
        };
        target.draw(
            &self.debug_overlay_buffer,
            NoIndices(glium::index::PrimitiveType::TrianglesList),
            &self.debug_program,
            &uniform! { u_screen: [screen_width as f32, screen_height as f32] },
            &params,
        ).map_err(|error| RendererError::new(format!("Debug overlay draw: {error}")))
    }

    #[cfg(target_os = "android")]
    fn draw_mobile_controls(
        &mut self,
        target: &mut glium::Frame,
        state: MobileControlsState,
        settings: RenderSettings,
    ) -> Result<(), RendererError> {
        let (screen_width, screen_height) = self.display.get_framebuffer_dimensions();
        let layout = MobileControlsLayout::new(screen_width, screen_height);
        self.mobile_controls_vertices.clear();
        self.mobile_controls_vertices.extend(build_mobile_controls_vertices(
            layout, state, settings, self.debug_visible,
        ));

        if self.mobile_controls_vertices.len() != self.mobile_controls_buffer.len() {
            return Err(RendererError::new("Mobile HUD vertex count changed unexpectedly"));
        }
        self.mobile_controls_buffer.write(&self.mobile_controls_vertices);

        let params = DrawParameters {
            blend: Blend::alpha_blending(),
            ..DrawParameters::default()
        };
        target.draw(
            &self.mobile_controls_buffer,
            NoIndices(glium::index::PrimitiveType::TrianglesList),
            &self.debug_program,
            &uniform! { u_screen: [screen_width as f32, screen_height as f32] },
            &params,
        ).map_err(|error| RendererError::new(format!("Mobile controls draw: {error}")))
    }

}

impl RendererBackend for OpenGlRenderer {
    fn info(&self) -> RendererInfo {
        #[cfg(target_os = "android")]
        let (name, api, api_version) = (
            "Booking Mobile Renderer",
            GraphicsApi::OpenGlEs,
            "OpenGL ES 3.0 / GLSL ES 3.00",
        );
        #[cfg(not(target_os = "android"))]
        let (name, api, api_version) = (
            "Booking Desktop Renderer",
            GraphicsApi::OpenGl,
            "OpenGL via Glium/Glutin",
        );
        RendererInfo {
            name: name.to_owned(),
            api,
            api_version: api_version.to_owned(),
        }
    }

    fn render(&mut self, frame: &RenderFrame) -> Result<(), RendererError> {
        self.update_frame_stats();
        let mut target = self.display.draw();
        let clear = frame.clear_color;
        target.clear_color_and_depth((clear.red, clear.green, clear.blue, clear.alpha), 1.0);
        let parameters = DrawParameters {
            depth: Depth { test: DepthTest::IfLess, write: true, ..Depth::default() },
            ..DrawParameters::default()
        };
        let (width, height) = self.display.get_framebuffer_dimensions();
        let view_projection = voxel_camera_matrix(width as f32 / height.max(1) as f32, frame.camera);
        let render_distance = frame.settings.render_distance_chunks as f32 * 16.0;
        let render_distance_squared = render_distance * render_distance;
        self.visible_chunk_count = 0;
        for chunk in &self.chunk_meshes {
            let center_x = chunk.chunk_x as f32 * 16.0 + 8.0;
            let center_z = chunk.chunk_z as f32 * 16.0 + 8.0;
            let dx = center_x - frame.camera.position[0];
            let dz = center_z - frame.camera.position[2];
            if dx * dx + dz * dz > render_distance_squared {
                continue;
            }
            self.visible_chunk_count += 1;
            target.draw(
                &chunk.vertices,
                NoIndices(glium::index::PrimitiveType::TrianglesList),
                &self.program,
                &uniform! { u_view_projection: view_projection, u_atlas: self.atlas.sampled()
                    .minify_filter(MinifySamplerFilter::Nearest)
                    .magnify_filter(MagnifySamplerFilter::Nearest) },
                &parameters,
            ).map_err(|error| RendererError::new(format!(
                "OpenGL chunk ({}, {}) draw: {error}", chunk.chunk_x, chunk.chunk_z
            )))?;
        }
        if self.debug_visible { self.draw_debug_overlay(&mut target)?; }
        #[cfg(target_os = "android")]
        self.draw_mobile_controls(&mut target, frame.mobile_controls, frame.settings)?;
        target.finish().map_err(|error| RendererError::new(format!("OpenGL present: {error}")))?;
        #[cfg(target_os = "android")]
        if !self.first_frame_logged {
            self.first_frame_logged = true;
            record_android_diagnostic("Renderer: first frame presented successfully");
        }
        Ok(())
    }
}

fn normalize3(value: [f32; 3]) -> [f32; 3] {
    let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt().max(0.0001);
    [value[0] / length, value[1] / length, value[2] / length]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// Perspective camera aimed at the center of the procedural voxel showcase.
/// Matrices are column-major to match GLSL uniforms.
fn voxel_camera_matrix(aspect: f32, camera: CameraState) -> [[f32; 4]; 4] {
    let eye = camera.position;
    let forward = normalize3(camera.forward());
    let side = normalize3(cross3(forward, [0.0, 1.0, 0.0]));
    let up = cross3(side, forward);
    let view = [
        [side[0], up[0], -forward[0], 0.0],
        [side[1], up[1], -forward[1], 0.0],
        [side[2], up[2], -forward[2], 0.0],
        [-dot3(side, eye), -dot3(up, eye), dot3(forward, eye), 1.0],
    ];
    let near = camera.near_plane.max(0.01);
    let far = camera.far_plane.max(near + 1.0);
    let focal = 1.0 / (camera.vertical_fov_radians.clamp(0.2, 2.8) * 0.5).tan();
    let projection = [
        [focal / aspect.max(0.1), 0.0, 0.0, 0.0],
        [0.0, focal, 0.0, 0.0],
        [0.0, 0.0, (far + near) / (near - far), -1.0],
        [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
    ];
    let mut result = [[0.0; 4]; 4];
    for column in 0..4 {
        for row in 0..4 {
            result[column][row] = (0..4).map(|index| projection[index][row] * view[column][index]).sum();
        }
    }
    result
}

#[cfg(test)]
mod camera_tests {
    use super::{voxel_camera_matrix, CameraState};

    #[test]
    fn camera_matrix_is_finite_for_mobile_aspect_ratios() {
        for aspect in [0.5, 1.0, 1.777, 2.5] {
            let matrix = voxel_camera_matrix(aspect, CameraState::default());
            assert!(matrix.iter().flatten().all(|value| value.is_finite()));
        }
    }
}

fn build_debug_overlay_vertices(
    screen_width: u32,
    _screen_height: u32,
    fps: u32,
    frame_ms: f32,
    visible_chunks: usize,
    vertex_count: usize,
) -> Vec<DebugVertex> {
    let lines = [
        "MINECRAFT BOOK EDITION 26.60".to_owned(),
        format!("FPS: {:>3}", fps.min(999)),
        format!("FRAME: {:>6.2} MS", frame_ms.clamp(0.0, 999.99)),
        "RENDERER: BOOKING".to_owned(),
        "API: OPENGL ES / OPENGL".to_owned(),
        format!("CHUNKS: {:>3} / {:>3}", visible_chunks.min(999), 126),
        "BLOCKS: PROCEDURAL WORLD".to_owned(),
        format!("VERTICES: {:>9}", vertex_count.min(999_999_999)),
        "F3: HIDE DEBUG".to_owned(),
    ];
    let scale = 2.0_f32;
    let char_width = 12.0;
    let line_height = 18.0;
    let panel_width = lines.iter()
        .map(|line| line.chars().count() as f32 * char_width)
        .fold(0.0_f32, f32::max) + 20.0;
    let panel_height = lines.len() as f32 * line_height + 18.0;
    let mut vertices = Vec::with_capacity(8_000);
    push_rect(&mut vertices, 10.0, 10.0, panel_width.min(screen_width as f32 - 20.0), panel_height,
              [0.015, 0.02, 0.025, 0.78]);
    for (line_index, line) in lines.iter().enumerate() {
        let y = 19.0 + line_index as f32 * line_height;
        let mut x = 19.0;
        for character in line.chars() {
            draw_character(&mut vertices, character, x, y, scale);
            x += char_width;
        }
    }
    vertices
}

#[cfg(target_os = "android")]
fn build_mobile_controls_vertices(
    layout: MobileControlsLayout,
    state: MobileControlsState,
    settings: RenderSettings,
    debug_visible: bool,
) -> Vec<DebugVertex> {
    let mut vertices = Vec::with_capacity(8_000);
    push_pixel_disc(
        &mut vertices,
        layout.joystick_center,
        layout.joystick_radius,
        state.strafe.abs() > 0.02 || state.forward.abs() > 0.02,
        [state.strafe, state.forward],
    );

    push_mobile_button(&mut vertices, layout.jump, "PULO", state.jump_pressed);
    push_mobile_button(&mut vertices, layout.sprint, "CORRER", state.sprint_pressed);
    push_mobile_button(&mut vertices, layout.break_block, "MINAR", state.break_pressed);
    push_mobile_button(&mut vertices, layout.place_block, "COLOCAR", state.place_pressed);
    push_mobile_button(&mut vertices, layout.debug, "DBG", debug_visible);
    push_mobile_button(&mut vertices, layout.settings, "CFG", settings.menu_open);

    push_settings_menu(&mut vertices, layout, settings);
    vertices
}

#[cfg(target_os = "android")]
fn push_settings_menu(
    vertices: &mut Vec<DebugVertex>,
    layout: MobileControlsLayout,
    settings: RenderSettings,
) {
    let visible = settings.menu_open;
    let panel = layout.settings_panel;
    let alpha = if visible { 0.97 } else { 0.0 };
    push_rect(vertices, panel.x, panel.y, panel.width, panel.height,
        [0.015, 0.02, 0.025, alpha]);
    push_rect(vertices, panel.x + 5.0, panel.y + 5.0, panel.width - 10.0, panel.height - 10.0,
        [0.13, 0.15, 0.17, if visible { 0.98 } else { 0.0 }]);
    let text_color = if visible { [0.94, 0.97, 1.0, 1.0] } else { [0.0, 0.0, 0.0, 0.0] };
    draw_text_colored(vertices, "CONFIGURACOES DE TESTE",
        panel.x + 22.0, panel.y + 28.0, 2.3, text_color);
    push_menu_option(vertices, layout.settings_close, "X", false, visible);

    draw_text_colored(vertices, "LIMITE DE FPS",
        panel.x + 22.0, panel.y + panel.height * 0.29, 1.8, text_color);
    let fps_labels = ["30 FPS", "60 FPS", "90 FPS", "SEM LIMITE"];
    for index in 0..layout.fps_options.len() {
        let selected = settings.fps_limit == RenderSettings::FPS_OPTIONS[index];
        push_menu_option(vertices, layout.fps_options[index], fps_labels[index], selected, visible);
    }

    draw_text_colored(vertices, "DISTANCIA DE RENDERIZACAO",
        panel.x + 22.0, panel.y + panel.height * 0.59, 1.8, text_color);
    for index in 0..layout.distance_options.len() {
        let value = RenderSettings::DISTANCE_OPTIONS[index];
        let label = format!("{value} CHUNKS");
        let selected = settings.render_distance_chunks == value;
        push_menu_option(vertices, layout.distance_options[index], &label, selected, visible);
    }
    if visible {
        draw_text_colored(vertices, "MAIOR DISTANCIA = MAIS CUSTO NA GPU",
            panel.x + 22.0, panel.y + panel.height - 28.0, 1.15, text_color);
    } else {
        // Keep a fixed vertex count while the panel is closed, so the persistent
        // GPU buffer never needs to be resized during gameplay.
        draw_text_colored(vertices, "MAIOR DISTANCIA = MAIS CUSTO NA GPU",
            panel.x + 22.0, panel.y + panel.height - 28.0, 1.15, [0.0, 0.0, 0.0, 0.0]);
    }
}

#[cfg(target_os = "android")]
fn push_menu_option(
    vertices: &mut Vec<DebugVertex>,
    rect: book_renderer_api::TouchRect,
    label: &str,
    selected: bool,
    visible: bool,
) {
    let alpha = if visible { 0.96 } else { 0.0 };
    push_rect(vertices, rect.x, rect.y, rect.width, rect.height,
        [0.025, 0.035, 0.045, alpha]);
    push_rect(vertices, rect.x + 3.0, rect.y + 3.0, rect.width - 6.0, rect.height - 6.0,
        if visible && selected { [0.22, 0.38, 0.16, 0.98] }
        else if visible { [0.28, 0.31, 0.34, 0.98] }
        else { [0.0, 0.0, 0.0, 0.0] });
    let color = if visible { [0.95, 0.97, 1.0, 1.0] } else { [0.0, 0.0, 0.0, 0.0] };
    let scale = ((rect.width - 12.0) / (label.chars().count() as f32 * 6.0)).clamp(0.7, 1.25);
    let text_width = label.chars().count() as f32 * 6.0 * scale;
    draw_text_colored(vertices, label,
        rect.x + (rect.width - text_width) * 0.5,
        rect.y + (rect.height - 7.0 * scale) * 0.5,
        scale,
        color);
}

fn draw_text_colored(
    vertices: &mut Vec<DebugVertex>,
    text: &str,
    mut x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
) {
    for character in text.chars() {
        draw_character_colored(vertices, character, x, y, scale, color);
        x += 6.0 * scale;
    }
}

fn draw_character_colored(
    vertices: &mut Vec<DebugVertex>,
    character: char,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
) {
    let rows = glyph(character);
    for (row_index, row) in rows.iter().enumerate() {
        for column in 0..5 {
            if row & (1 << (4 - column)) != 0 {
                push_rect(vertices, x + column as f32 * scale,
                    y + row_index as f32 * scale, scale, scale, color);
            }
        }
    }
}

fn push_pixel_disc(
    vertices: &mut Vec<DebugVertex>,
    center: [f32; 2],
    radius: f32,
    pressed: bool,
    movement: [f32; 2],
) {
    let cells = 13_i32;
    let cell = radius * 2.0 / cells as f32;
    for y in 0..cells {
        for x in 0..cells {
            let nx = (x as f32 + 0.5) / cells as f32 * 2.0 - 1.0;
            let ny = (y as f32 + 0.5) / cells as f32 * 2.0 - 1.0;
            let distance = (nx * nx + ny * ny).sqrt();
            if distance > 1.0 { continue; }
            let shade = ((x * 7 + y * 11 + (x * y) % 5) % 4) as f32;
            let color = if distance > 0.78 {
                [0.08 + shade * 0.018, 0.09 + shade * 0.018, 0.10 + shade * 0.018, 0.78]
            } else {
                [0.16 + shade * 0.018, 0.18 + shade * 0.018, 0.20 + shade * 0.018, 0.48]
            };
            push_rect(
                vertices,
                center[0] - radius + x as f32 * cell,
                center[1] - radius + y as f32 * cell,
                cell + 0.5,
                cell + 0.5,
                color,
            );
        }
    }

    let thumb_center = [
        center[0] + movement[0].clamp(-1.0, 1.0) * radius * 0.48,
        center[1] - movement[1].clamp(-1.0, 1.0) * radius * 0.48,
    ];
    let thumb_radius = radius * 0.38;
    let thumb_cells = 9_i32;
    let thumb_cell = thumb_radius * 2.0 / thumb_cells as f32;
    for y in 0..thumb_cells {
        for x in 0..thumb_cells {
            let nx = (x as f32 + 0.5) / thumb_cells as f32 * 2.0 - 1.0;
            let ny = (y as f32 + 0.5) / thumb_cells as f32 * 2.0 - 1.0;
            if nx * nx + ny * ny > 1.0 { continue; }
            let shade = ((x * 3 + y * 5) % 3) as f32 * 0.035;
            let base = if pressed { 0.48 } else { 0.34 };
            push_rect(
                vertices,
                thumb_center[0] - thumb_radius + x as f32 * thumb_cell,
                thumb_center[1] - thumb_radius + y as f32 * thumb_cell,
                thumb_cell + 0.5,
                thumb_cell + 0.5,
                [base + shade, base + shade, base + shade, 0.94],
            );
        }
    }
}

fn push_mobile_button(
    vertices: &mut Vec<DebugVertex>,
    rect: MobileControlsLayoutRect,
    label: &str,
    pressed: bool,
) {
    let x = rect.x;
    let y = rect.y;
    let width = rect.width;
    let height = rect.height;
    let outer = if pressed { [0.04, 0.09, 0.035, 0.96] } else { [0.025, 0.03, 0.035, 0.92] };
    let top = if pressed { [0.62, 0.72, 0.40, 0.98] } else { [0.58, 0.62, 0.66, 0.96] };
    push_rect(vertices, x, y, width, height, outer);
    push_rect(vertices, x + 3.0, y + 3.0, width - 6.0, height - 6.0, top);
    push_rect(
        vertices, x + 5.0, y + 5.0, width - 10.0, height - 10.0,
        if pressed { [0.20, 0.34, 0.13, 0.98] } else { [0.23, 0.25, 0.27, 0.98] },
    );

    // Small irregular stone pixels give the temporary HUD a Bedrock-like
    // block texture without embedding Mojang's copyrighted UI sprites.
    let tile = 9.0_f32;
    let shades: [[f32; 4]; 5] = [
        [0.32, 0.34, 0.36, 0.86],
        [0.39, 0.41, 0.43, 0.86],
        [0.27, 0.29, 0.31, 0.86],
        [0.44, 0.46, 0.48, 0.86],
        [0.35, 0.37, 0.39, 0.86],
    ];
    let mut row = 0_i32;
    let mut py = y + 7.0;
    while py < y + height - 7.0 {
        let mut column = 0_i32;
        let mut px = x + 7.0;
        while px < x + width - 7.0 {
            let index = ((column * 7 + row * 11 + (column * row) % 5).unsigned_abs() as usize) % shades.len();
            let mut color = shades[index];
            if pressed {
                color[0] = (color[0] * 0.82).min(1.0);
                color[1] = (color[1] * 1.08).min(1.0);
            }
            push_rect(vertices, px, py, tile - 0.5, tile - 0.5, color);
            px += tile;
            column += 1;
        }
        py += tile;
        row += 1;
    }

    let scale = ((width - 12.0) / (label.chars().count() as f32 * 6.0)).clamp(0.85, 1.6);
    let label_width = label.chars().count() as f32 * 6.0 * scale;
    let mut text_x = x + (width - label_width) * 0.5;
    let text_y = y + (height - 7.0 * scale) * 0.5;
    for character in label.chars() {
        draw_character(vertices, character, text_x, text_y, scale);
        text_x += 6.0 * scale;
    }
}

/// Local alias keeps the helper signature compact while using the shared hit layout.
type MobileControlsLayoutRect = book_renderer_api::TouchRect;

fn push_rect(vertices: &mut Vec<DebugVertex>, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
    let p0 = [x, y];
    let p1 = [x + width, y];
    let p2 = [x + width, y + height];
    let p3 = [x, y + height];
    for position in [p0, p1, p2, p0, p2, p3] {
        vertices.push(DebugVertex { position, color });
    }
}

fn draw_character(vertices: &mut Vec<DebugVertex>, character: char, x: f32, y: f32, scale: f32) {
    let rows = glyph(character);
    let white = [0.92, 0.96, 1.0, 1.0];
    for (row_index, row) in rows.iter().enumerate() {
        for column in 0..5 {
            if row & (1 << (4 - column)) != 0 {
                push_rect(vertices, x + column as f32 * scale,
                    y + row_index as f32 * scale, scale, scale, white);
            }
        }
    }
}

fn glyph(character: char) -> [u8; 7] {
    match character {
        'A' => [14,17,17,31,17,17,17], 'B' => [30,17,17,30,17,17,30],
        'C' => [14,17,16,16,16,17,14], 'D' => [30,17,17,17,17,17,30],
        'E' => [31,16,16,30,16,16,31], 'F' => [31,16,16,30,16,16,16],
        'G' => [14,17,16,23,17,17,15], 'H' => [17,17,17,31,17,17,17],
        'I' => [14,4,4,4,4,4,14], 'J' => [7,2,2,2,18,18,12],
        'K' => [17,18,20,24,20,18,17], 'L' => [16,16,16,16,16,16,31],
        'M' => [17,27,21,21,17,17,17], 'N' => [17,25,21,19,17,17,17],
        'O' => [14,17,17,17,17,17,14], 'P' => [30,17,17,30,16,16,16],
        'Q' => [14,17,17,17,21,18,13], 'R' => [30,17,17,30,20,18,17],
        'S' => [15,16,16,14,1,1,30], 'T' => [31,4,4,4,4,4,4],
        'U' => [17,17,17,17,17,17,14], 'V' => [17,17,17,17,17,10,4],
        'W' => [17,17,17,21,21,21,10], 'X' => [17,17,10,4,10,17,17],
        'Y' => [17,17,10,4,4,4,4], 'Z' => [31,1,2,4,8,16,31],
        '0' => [14,17,19,21,25,17,14], '1' => [4,12,4,4,4,4,14],
        '2' => [14,17,1,2,4,8,31], '3' => [30,1,1,14,1,1,30],
        '4' => [2,6,10,18,31,2,2], '5' => [31,16,16,30,1,1,30],
        '6' => [14,16,16,30,17,17,14], '7' => [31,1,2,4,8,8,8],
        '8' => [14,17,17,14,17,17,14], '9' => [14,17,17,15,1,1,14],
        ':' => [0,4,4,0,4,4,0], '.' => [0,0,0,0,0,12,12],
        ',' => [0,0,0,0,4,4,8], '-' => [0,0,0,31,0,0,0],
        '/' => [1,2,2,4,8,8,16], '(' => [2,4,8,8,8,4,2],
        ')' => [8,4,2,2,2,4,8], _ => [0;7],
    }
}


#[cfg(target_os = "android")]
#[allow(unsafe_code)]
fn create_android_display(
    event_loop: &glium::winit::event_loop::ActiveEventLoop,
    title: &str,
) -> Result<
    (
        glium::winit::window::Window,
        glium::backend::glutin::Display<WindowSurface>,
    ),
    RendererError,
> {
    use glium::winit::window::Window;

    record_android_diagnostic("EGL: preparing Android native window and display");
    let attributes = Window::default_attributes().with_title(title);
    let display_builder = DisplayBuilder::new().with_window_attributes(Some(attributes));
    // 16-bit depth buffers are broadly supported by mobile EGL implementations.
    let template = ConfigTemplateBuilder::new()
        .with_api(Api::GLES3)
        .with_depth_size(16);
    record_android_diagnostic("EGL: requesting an OpenGL ES 3 configuration");
    let (window, config) = display_builder
        .build(event_loop, template, |mut configs| {
            let selected = configs.next();
            record_android_diagnostic(&format!(
                "EGL: OpenGL ES 3 configuration available: {}",
                selected.is_some()
            ));
            selected.unwrap_or_else(|| {
                record_android_diagnostic(
                    "EGL ERROR: no OpenGL ES 3 EGL configuration was returned",
                );
                panic!("no OpenGL ES 3 EGL configuration found");
            })
        })
        .map_err(|error| RendererError::new(format!("Android EGL configuration: {error}")))?;
    record_android_diagnostic("EGL: configuration created");
    let window = window.ok_or_else(|| RendererError::new("Android native window was not created"))?;
    record_android_diagnostic("Android: native window created");
    let raw_window = window
        .window_handle()
        .map_err(|error| RendererError::new(format!("Android window handle: {error}")))?
        .as_raw();
    record_android_diagnostic("EGL: requesting an OpenGL ES 3.0 context");
    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(Some(Version::new(3, 0))))
        .build(Some(raw_window));
    let not_current = unsafe {
        config
            .display()
            .create_context(&config, &context_attributes)
    }
    .map_err(|error| RendererError::new(format!("OpenGL ES 3 context: {error}")))?;
    record_android_diagnostic("EGL: OpenGL ES 3.0 context created");

    let size = window.inner_size();
    let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window,
        NonZeroU32::new(size.width.max(1)).expect("non-zero surface width"),
        NonZeroU32::new(size.height.max(1)).expect("non-zero surface height"),
    );
    record_android_diagnostic("EGL: creating Android window surface");
    let surface = unsafe {
        config
            .display()
            .create_window_surface(&config, &surface_attributes)
    }
    .map_err(|error| RendererError::new(format!("Android EGL surface: {error}")))?;
    record_android_diagnostic("EGL: Android window surface created");
    record_android_diagnostic("EGL: making OpenGL ES context current");
    let current = not_current
        .make_current(&surface)
        .map_err(|error| RendererError::new(format!("Make OpenGL ES current: {error}")))?;
    record_android_diagnostic("EGL: OpenGL ES context is current");
    let display = glium::backend::glutin::Display::from_context_surface(current, surface)
        .map_err(|error| RendererError::new(format!("Glium display: {error}")))?;
    record_android_diagnostic("Glium: display initialized");

    Ok((window, display))
}

fn upload_chunk_meshes(
    display: &glium::backend::glutin::Display<WindowSurface>,
    engine: &BookingEngine,
) -> Result<Vec<ChunkGpuMesh>, RendererError> {
    let mut meshes = Vec::new();
    for (chunk_x, chunk_z, vertices) in sample_world_meshes(engine) {
        if vertices.is_empty() { continue; }
        let buffer = VertexBuffer::new(display, &vertices)
            .map_err(|error| RendererError::new(format!(
                "Chunk ({chunk_x}, {chunk_z}) vertex buffer: {error}"
            )))?;
        meshes.push(ChunkGpuMesh { chunk_x, chunk_z, vertices: buffer });
    }
    Ok(meshes)
}

fn upload_chunk_meshes_for_chunks(
    display: &glium::backend::glutin::Display<WindowSurface>,
    engine: &BookingEngine,
    positions: &BTreeSet<(i32, i32)>,
) -> Result<Vec<ChunkGpuMesh>, RendererError> {
    let mut meshes = Vec::new();
    for (chunk_x, chunk_z, vertices) in sample_world_meshes_for_chunks(engine, positions) {
        if vertices.is_empty() { continue; }
        let buffer = VertexBuffer::new(display, &vertices)
            .map_err(|error| RendererError::new(format!(
                "Chunk ({chunk_x}, {chunk_z}) vertex buffer: {error}"
            )))?;
        meshes.push(ChunkGpuMesh { chunk_x, chunk_z, vertices: buffer });
    }
    Ok(meshes)
}

fn sample_world_meshes(engine: &BookingEngine) -> Vec<(i32, i32, Vec<Vertex>)> {
    // Merge every vertical section into one GPU buffer per chunk column.
    // This reduces draw calls substantially on mobile GPUs.
    let mut chunk_vertices: BTreeMap<(i32, i32), Vec<Vertex>> = BTreeMap::new();
    for section in engine.build_loaded_world_meshes() {
        let offset_x = section.position.x as f32 * 16.0;
        let offset_z = section.position.z as f32 * 16.0;
        let offset_y = section.section_y as f32 * 16.0;
        let vertices = chunk_vertices
            .entry((section.position.x, section.position.z))
            .or_default();
        for vertex in section.mesh.vertices {
            let color = match vertex.block_id {
                BlockId(1) => [0.30, 0.68, 0.24],
                BlockId(2) => [0.48, 0.31, 0.18],
                BlockId(3) => [0.48, 0.50, 0.52],
                BlockId(4) => [0.24, 0.25, 0.27],
                _ => [1.0, 0.0, 1.0],
            };
            vertices.push(Vertex {
                position: [
                    vertex.position[0] + offset_x,
                    vertex.position[1] + offset_y,
                    vertex.position[2] + offset_z,
                ],
                uv: vertex.uv,
                normal: vertex.normal,
                color,
            });
        }
    }
    chunk_vertices.into_iter()
        .map(|((chunk_x, chunk_z), vertices)| (chunk_x, chunk_z, vertices))
        .collect()
}

fn sample_world_meshes_for_chunks(
    engine: &BookingEngine,
    positions: &BTreeSet<(i32, i32)>,
) -> Vec<(i32, i32, Vec<Vertex>)> {
    let mut chunk_vertices: BTreeMap<(i32, i32), Vec<Vertex>> = BTreeMap::new();
    for section in engine.build_world_meshes_for_chunks(positions) {
        let offset_x = section.position.x as f32 * 16.0;
        let offset_z = section.position.z as f32 * 16.0;
        let offset_y = section.section_y as f32 * 16.0;
        let vertices = chunk_vertices
            .entry((section.position.x, section.position.z))
            .or_default();
        for vertex in section.mesh.vertices {
            let color = match vertex.block_id {
                BlockId(1) => [0.30, 0.68, 0.24],
                BlockId(2) => [0.48, 0.31, 0.18],
                BlockId(3) => [0.48, 0.50, 0.52],
                BlockId(4) => [0.24, 0.25, 0.27],
                _ => [1.0, 0.0, 1.0],
            };
            vertices.push(Vertex {
                position: [
                    vertex.position[0] + offset_x,
                    vertex.position[1] + offset_y,
                    vertex.position[2] + offset_z,
                ],
                uv: vertex.uv,
                normal: vertex.normal,
                color,
            });
        }
    }
    chunk_vertices.into_iter()
        .map(|((chunk_x, chunk_z), vertices)| (chunk_x, chunk_z, vertices))
        .collect()
}

fn prototype_texture_atlas() -> TextureAtlas {
    fn image(key: &str, base: [u8; 3], seed: u32) -> TextureImage {
        let mut rgba = Vec::with_capacity(16 * 16 * 4);
        for y in 0..16_u32 {
            for x in 0..16_u32 {
                let variation = ((x * 7 + y * 11 + seed * 13) % 17) as i16 - 8;
                for channel in base {
                    rgba.push((channel as i16 + variation).clamp(0, 255) as u8);
                }
                rgba.push(255);
            }
        }
        TextureImage {
            key: key.to_owned(), width: 16, height: 16, rgba,
        }
    }

    TextureAtlas::build_grid(
        vec![
            image("book:grass_block", [72, 148, 54], 1),
            image("book:dirt", [116, 76, 45], 2),
            image("book:stone", [126, 130, 134], 3),
        ],
        16,
        16,
    ).expect("built-in prototype texture atlas must be valid")
}
