use book_core::{AtlasTile, BlockDefinition, BlockId, BlockRegistry};
use book_meshing::mesh_section;
use book_renderer_api::{GraphicsApi, RendererBackend, RendererError, RendererInfo, RenderFrame};
use book_world::{ChunkSection, SECTION_EDGE};
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
const VERTEX_SHADER: &str = r#"
#version 300 es
precision highp float;
in vec3 position;
in vec2 uv;
out vec2 v_uv;
uniform float u_time;
void main() {
    vec3 centered = position - vec3(8.0, 0.0, 8.0);
    float angle = 0.42 + sin(u_time * 0.12) * 0.08;
    float c = cos(angle);
    float s = sin(angle);
    float rotated_x = centered.x * c - centered.z * s;
    float rotated_z = centered.x * s + centered.z * c;
    gl_Position = vec4(rotated_x * 0.075, centered.y * 0.13 - rotated_z * 0.045,
                       0.5 + rotated_z * 0.001 - centered.y * 0.001, 1.0);
    v_uv = uv;
}
"#;

#[cfg(target_os = "android")]
const FRAGMENT_SHADER: &str = r#"
#version 300 es
precision highp float;
in vec2 v_uv;
out vec4 out_color;
void main() {
    float texture_hint = 0.94 + fract(v_uv.x * 16.0) * 0.06;
    out_color = vec4(vec3(0.30, 0.58, 0.25) * texture_hint, 1.0);
}
"#;

#[cfg(target_os = "android")]
const DEBUG_VERTEX_SHADER: &str = r#"
#version 300 es
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
const DEBUG_FRAGMENT_SHADER: &str = r#"
#version 300 es
precision highp float;
in vec4 v_color;
out vec4 out_color;
void main() { out_color = v_color; }
"#;

#[derive(Clone, Copy)]
struct Vertex { position: [f32; 3], uv: [f32; 2] }
implement_vertex!(Vertex, position, uv);

#[derive(Clone, Copy)]
struct DebugVertex { position: [f32; 2], color: [f32; 4] }
implement_vertex!(DebugVertex, position, color);

#[cfg(not(target_os = "android"))]
const VERTEX_SHADER: &str = r#"
#version 140
in vec3 position;
in vec2 uv;
out vec2 v_uv;
uniform float u_time;
void main() {
    vec3 centered = position - vec3(8.0, 0.0, 8.0);
    float angle = 0.42 + sin(u_time * 0.12) * 0.08;
    float c = cos(angle);
    float s = sin(angle);
    float rotated_x = centered.x * c - centered.z * s;
    float rotated_z = centered.x * s + centered.z * c;
    gl_Position = vec4(rotated_x * 0.075, centered.y * 0.13 - rotated_z * 0.045,
                       0.5 + rotated_z * 0.001 - centered.y * 0.001, 1.0);
    v_uv = uv;
}
"#;

#[cfg(not(target_os = "android"))]
const FRAGMENT_SHADER: &str = r#"
#version 140
in vec2 v_uv;
out vec4 out_color;
void main() {
    float texture_hint = 0.94 + fract(v_uv.x * 16.0) * 0.06;
    out_color = vec4(vec3(0.30, 0.58, 0.25) * texture_hint, 1.0);
}
"#;

#[cfg(not(target_os = "android"))]
const DEBUG_VERTEX_SHADER: &str = r#"
#version 140
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
const DEBUG_FRAGMENT_SHADER: &str = r#"
#version 140
in vec4 v_color;
out vec4 out_color;
void main() { out_color = v_color; }
"#;

pub struct OpenGlRenderer {
    display: glium::backend::glutin::Display<WindowSurface>,
    chunk_vertices: VertexBuffer<Vertex>,
    program: Program,
    debug_program: Program,
    started: Instant,
    last_frame: Instant,
    fps_window_started: Instant,
    frames_in_window: u32,
    fps: u32,
    last_frame_ms: f32,
    debug_visible: bool,
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
        width: u32,
        height: u32,
    ) -> Result<(glium::winit::window::Window, Self), RendererError> {
        let (window, display) = create_android_display(event_loop, title, width, height)?;
        Self::from_display(window, display)
    }

    fn from_display(
        window: glium::winit::window::Window,
        display: glium::backend::glutin::Display<WindowSurface>,
    ) -> Result<(glium::winit::window::Window, Self), RendererError> {
        let chunk_vertices = VertexBuffer::new(&display, &demo_chunk_vertices())
            .map_err(|error| RendererError::new(format!("Chunk vertex buffer: {error}")))?;
        let program = Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None)
            .map_err(|error| RendererError::new(format!("Chunk shader program: {error}")))?;
        let debug_program = Program::from_source(
            &display, DEBUG_VERTEX_SHADER, DEBUG_FRAGMENT_SHADER, None,
        ).map_err(|error| RendererError::new(format!("Debug overlay shader: {error}")))?;
        let now = Instant::now();
        Ok((window, Self {
            display, chunk_vertices, program, debug_program,
            started: now, last_frame: now, fps_window_started: now,
            frames_in_window: 0, fps: 0, last_frame_ms: 0.0, debug_visible: false,
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
        let vertex_count = self.chunk_vertices.len();
        let lines = [
            "MINECRAFT BOOK EDITION 26.60".to_owned(),
            format!("FPS: {}", self.fps),
            format!("FRAME: {:.2} MS", self.last_frame_ms),
            "RENDERER: OPENGL".to_owned(),
            "API: OPENGL".to_owned(),
            "CHUNKS: 1 (TEST)".to_owned(),
            "BLOCKS: 256 (TEST)".to_owned(),
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
        let (screen_width, _screen_height) = self.display.get_framebuffer_dimensions();
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
        let elapsed = self.started.elapsed().as_secs_f32();
        target.draw(
            &self.chunk_vertices,
            NoIndices(glium::index::PrimitiveType::TrianglesList),
            &self.program,
            &uniform! { u_time: elapsed },
            &parameters,
        ).map_err(|error| RendererError::new(format!("OpenGL chunk draw: {error}")))?;
        if self.debug_visible { self.draw_debug_overlay(&mut target)?; }
        #[cfg(target_os = "android")]
        self.draw_debug_button(&mut target)?;
        target.finish().map_err(|error| RendererError::new(format!("OpenGL present: {error}")))
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
    width: u32,
    height: u32,
) -> Result<
    (
        glium::winit::window::Window,
        glium::backend::glutin::Display<WindowSurface>,
    ),
    RendererError,
> {
    use glium::winit::dpi::PhysicalSize;
    use glium::winit::window::Window;

    let attributes = Window::default_attributes()
        .with_title(title)
        .with_inner_size(PhysicalSize::new(width, height));
    let display_builder = DisplayBuilder::new().with_window_attributes(Some(attributes));
    let template = ConfigTemplateBuilder::new()
        .with_api(Api::GLES3)
        .with_depth_size(24);
    let (window, config) = display_builder
        .build(event_loop, template, |mut configs| {
            configs.next().expect("no OpenGL ES 3 configuration found")
        })
        .map_err(|error| RendererError::new(format!("Android EGL configuration: {error}")))?;
    let window = window.ok_or_else(|| RendererError::new("Android window was not created"))?;
    let raw_window = window
        .window_handle()
        .map_err(|error| RendererError::new(format!("Android window handle: {error}")))?
        .as_raw();
    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(Some(Version::new(3, 0))))
        .build(Some(raw_window));
    let not_current = unsafe {
        config
            .display()
            .create_context(&config, &context_attributes)
    }
    .map_err(|error| RendererError::new(format!("OpenGL ES 3 context: {error}")))?;

    let size = window.inner_size();
    let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window,
        NonZeroU32::new(size.width.max(1)).expect("non-zero surface width"),
        NonZeroU32::new(size.height.max(1)).expect("non-zero surface height"),
    );
    let surface = unsafe {
        config
            .display()
            .create_window_surface(&config, &surface_attributes)
    }
    .map_err(|error| RendererError::new(format!("Android EGL surface: {error}")))?;
    let current = not_current
        .make_current(&surface)
        .map_err(|error| RendererError::new(format!("Make OpenGL ES current: {error}")))?;
    let display = glium::backend::glutin::Display::from_context_surface(current, surface)
        .map_err(|error| RendererError::new(format!("Glium display: {error}")))?;

    Ok((window, display))
}

fn demo_chunk_vertices() -> Vec<Vertex> {
    let mut registry = BlockRegistry::new();
    registry.register(BlockDefinition {
        id: BlockId(1), name: "book:grass_block".to_owned(),
        atlas_tile: AtlasTile { x: 0, y: 0 }, solid: true, opaque: true,
    });
    let mut section = ChunkSection::new();
    for z in 0..SECTION_EDGE {
        for x in 0..SECTION_EDGE { section.set(x, 0, z, BlockId(1)); }
    }
    mesh_section(&section, &registry, 16, 16).vertices.into_iter()
        .map(|vertex| Vertex { position: vertex.position, uv: vertex.uv })
        .collect()
}
