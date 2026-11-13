use booking_engine::BookingEngine;
use book_core::BlockId;
use book_renderer_api::{GraphicsApi, RendererBackend, RendererError, RendererInfo, RenderFrame};
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
use glium::uniform;
use glium::{implement_vertex, Blend, Depth, DepthTest, DrawParameters, Program, Surface, VertexBuffer};
use glutin::surface::WindowSurface;
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
out vec4 out_color;
void main() {
    vec3 sun_direction = normalize(vec3(-0.4, 0.8, -0.3));
    float diffuse = max(dot(normalize(v_normal), sun_direction), 0.0);
    float lighting = 0.42 + diffuse * 0.58;
    out_color = vec4(v_color * lighting, 1.0);
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
out vec4 out_color;
void main() {
    vec3 sun_direction = normalize(vec3(-0.4, 0.8, -0.3));
    float diffuse = max(dot(normalize(v_normal), sun_direction), 0.0);
    float lighting = 0.42 + diffuse * 0.58;
    out_color = vec4(v_color * lighting, 1.0);
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
    debug_program: Program,
    last_frame: Instant,
    fps_window_started: Instant,
    frames_in_window: u32,
    fps: u32,
    last_frame_ms: f32,
    debug_visible: bool,
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
    ) -> Result<(glium::winit::window::Window, Self), RendererError> {
        let (window, display) = SimpleWindowBuilder::new()
            .with_title(title)
            .with_inner_size(width, height)
            .build(event_loop);
        Self::from_display(window, display)
    }

    #[cfg(target_os = "android")]
    pub fn new(
        event_loop: &glium::winit::event_loop::ActiveEventLoop,
        title: &str,
        _width: u32,
        _height: u32,
    ) -> Result<(glium::winit::window::Window, Self), RendererError> {
        let (window, display) = create_android_display(event_loop, title)?;
        Self::from_display(window, display)
    }

    fn from_display(
        window: glium::winit::window::Window,
        display: glium::backend::glutin::Display<WindowSurface>,
    ) -> Result<(glium::winit::window::Window, Self), RendererError> {
        #[cfg(target_os = "android")]
        record_android_diagnostic("Renderer: building chunk-manager world mesh");
        let mut chunk_meshes = Vec::new();
        for (chunk_x, chunk_z, vertices) in sample_world_meshes() {
            if vertices.is_empty() { continue; }
            let buffer = VertexBuffer::new(&display, &vertices)
                .map_err(|error| RendererError::new(format!(
                    "Chunk ({chunk_x}, {chunk_z}) vertex buffer: {error}"
                )))?;
            chunk_meshes.push(ChunkGpuMesh {
                chunk_x, chunk_z, vertices: buffer,
            });
        }
        #[cfg(target_os = "android")]
        record_android_diagnostic("Renderer: compiling chunk shader program");
        let program = Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None)
            .map_err(|error| RendererError::new(format!("Chunk shader program: {error}")))?;
        #[cfg(target_os = "android")]
        record_android_diagnostic("Renderer: compiling debug overlay shader program");
        let debug_program = Program::from_source(
            &display, DEBUG_VERTEX_SHADER, DEBUG_FRAGMENT_SHADER, None,
        ).map_err(|error| RendererError::new(format!("Debug overlay shader: {error}")))?;
        #[cfg(target_os = "android")]
        record_android_diagnostic("Renderer: GPU resources and shader programs are ready");
        let now = Instant::now();
        Ok((window, Self {
            display, chunk_meshes, program, debug_program,
            last_frame: now, fps_window_started: now,
            frames_in_window: 0, fps: 0, last_frame_ms: 0.0, debug_visible: false,
            #[cfg(target_os = "android")]
            first_frame_logged: false,
        }))
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

    fn draw_debug_overlay(&self, target: &mut glium::Frame) -> Result<(), RendererError> {
        let vertex_count: usize = self.chunk_meshes.iter().map(|mesh| mesh.vertices.len()).sum();
        let lines = [
            "MINECRAFT BOOK EDITION 26.60".to_owned(),
            format!("FPS: {}", self.fps),
            format!("FRAME: {:.2} MS", self.last_frame_ms),
            "RENDERER: BOOKING".to_owned(),
            "API: OPENGL ES / OPENGL".to_owned(),
            format!("CHUNKS: {}", self.chunk_meshes.len()),
            "BLOCKS: PROCEDURAL WORLD".to_owned(),
            format!("VERTICES: {vertex_count}"),
            "F3: HIDE DEBUG".to_owned(),
        ];
        let scale = 2.0_f32;
        let char_width = 12.0;
        let line_height = 18.0;
        let panel_width = lines.iter()
            .map(|line| line.chars().count() as f32 * char_width)
            .fold(0.0_f32, f32::max) + 20.0;
        let panel_height = lines.len() as f32 * line_height + 18.0;
        let mut vertices = Vec::new();
        push_rect(&mut vertices, 10.0, 10.0, panel_width, panel_height,
                  [0.015, 0.02, 0.025, 0.78]);
        for (line_index, line) in lines.iter().enumerate() {
            let y = 19.0 + line_index as f32 * line_height;
            let mut x = 19.0;
            for character in line.chars() {
                draw_character(&mut vertices, character, x, y, scale);
                x += char_width;
            }
        }
        let (screen_width, screen_height) = self.display.get_framebuffer_dimensions();
        let buffer = VertexBuffer::new(&self.display, &vertices)
            .map_err(|error| RendererError::new(format!("Debug overlay buffer: {error}")))?;
        let params = DrawParameters { blend: Blend::alpha_blending(), ..DrawParameters::default() };
        target.draw(
            &buffer,
            NoIndices(glium::index::PrimitiveType::TrianglesList),
            &self.debug_program,
            &uniform! { u_screen: [screen_width as f32, screen_height as f32] },
            &params,
        ).map_err(|error| RendererError::new(format!("Debug overlay draw: {error}")))
    }

    #[cfg(target_os = "android")]
    fn draw_debug_button(&self, target: &mut glium::Frame) -> Result<(), RendererError> {
        let (screen_width, screen_height) = self.display.get_framebuffer_dimensions();
        let x = screen_width.saturating_sub(88) as f32;
        let mut vertices = Vec::new();
        push_rect(&mut vertices, x, 12.0, 76.0, 40.0, [0.015, 0.02, 0.025, 0.82]);
        for (index, character) in "DBG".chars().enumerate() {
            draw_character(&mut vertices, character, x + 12.0 + index as f32 * 16.0, 23.0, 2.0);
        }
        let buffer = VertexBuffer::new(&self.display, &vertices)
            .map_err(|error| RendererError::new(format!("Debug button buffer: {error}")))?;
        let params = DrawParameters { blend: Blend::alpha_blending(), ..DrawParameters::default() };
        target.draw(
            &buffer,
            NoIndices(glium::index::PrimitiveType::TrianglesList),
            &self.debug_program,
            &uniform! { u_screen: [screen_width as f32, screen_height as f32] },
            &params,
        ).map_err(|error| RendererError::new(format!("Debug button draw: {error}")))
    }
}

impl RendererBackend for OpenGlRenderer {
    fn info(&self) -> RendererInfo {
        RendererInfo {
            name: "Book OpenGL Chunk Renderer".to_owned(),
            api: GraphicsApi::OpenGl,
            api_version: "OpenGL via Glium/Glutin".to_owned(),
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
        let view_projection = voxel_camera_matrix(width as f32 / height.max(1) as f32);
        for chunk in &self.chunk_meshes {
            target.draw(
                &chunk.vertices,
                NoIndices(glium::index::PrimitiveType::TrianglesList),
                &self.program,
                &uniform! { u_view_projection: view_projection },
                &parameters,
            ).map_err(|error| RendererError::new(format!(
                "OpenGL chunk ({}, {}) draw: {error}", chunk.chunk_x, chunk.chunk_z
            )))?;
        }
        if self.debug_visible { self.draw_debug_overlay(&mut target)?; }
        #[cfg(target_os = "android")]
        self.draw_debug_button(&mut target)?;
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
fn voxel_camera_matrix(aspect: f32) -> [[f32; 4]; 4] {
    let eye = [39.0, 19.0, 43.0];
    let target = [24.0, 2.0, 24.0];
    let forward = normalize3([target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]]);
    let side = normalize3(cross3(forward, [0.0, 1.0, 0.0]));
    let up = cross3(side, forward);
    let view = [
        [side[0], up[0], -forward[0], 0.0],
        [side[1], up[1], -forward[1], 0.0],
        [side[2], up[2], -forward[2], 0.0],
        [-dot3(side, eye), -dot3(up, eye), dot3(forward, eye), 1.0],
    ];
    let near = 0.1_f32;
    let far = 128.0_f32;
    let focal = 1.0 / (std::f32::consts::FRAC_PI_3 * 0.5).tan();
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
    use super::voxel_camera_matrix;

    #[test]
    fn camera_matrix_is_finite_for_mobile_aspect_ratios() {
        for aspect in [0.5, 1.0, 1.777, 2.5] {
            let matrix = voxel_camera_matrix(aspect);
            assert!(matrix.iter().flatten().all(|value| value.is_finite()));
        }
    }
}

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

fn sample_world_meshes() -> Vec<(i32, i32, Vec<Vertex>)> {
    // Booking owns world generation and CPU meshing. The graphics backend
    // uploads one independent GPU buffer per loaded chunk section.
    let mut engine = BookingEngine::new();
    engine.generate_prototype_terrain(1);

    engine.build_loaded_world_meshes().into_iter().map(|section| {
        let offset_x = section.position.x as f32 * 16.0;
        let offset_z = section.position.z as f32 * 16.0;
        let offset_y = section.section_y as f32 * 16.0;
        let vertices = section.mesh.vertices.into_iter().map(|vertex| {
            let color = match vertex.block_id {
                BlockId(1) => [0.30, 0.68, 0.24],
                BlockId(2) => [0.48, 0.31, 0.18],
                BlockId(3) => [0.48, 0.50, 0.52],
                _ => [1.0, 0.0, 1.0],
            };
            Vertex {
                position: [
                    vertex.position[0] + offset_x,
                    vertex.position[1] + offset_y,
                    vertex.position[2] + offset_z,
                ],
                uv: vertex.uv,
                normal: vertex.normal,
                color,
            }
        }).collect();
        (section.position.x, section.position.z, vertices)
    }).collect()
}
