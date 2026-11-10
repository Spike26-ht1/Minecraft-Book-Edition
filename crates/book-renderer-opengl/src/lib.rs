use book_core::{AtlasTile, BlockDefinition, BlockId, BlockRegistry};
use book_meshing::mesh_section;
use book_renderer_api::{GraphicsApi, RendererBackend, RendererError, RendererInfo, RenderFrame};
use book_world::{ChunkSection, SECTION_EDGE};
use glium::backend::glutin::SimpleWindowBuilder;
use glium::index::NoIndices;
use glium::uniform;
use glium::{implement_vertex, Depth, DepthTest, DrawParameters, Program, Surface, VertexBuffer};
use glutin::surface::WindowSurface;
use std::time::Instant;

#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

implement_vertex!(Vertex, position, uv);

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

    float screen_x = rotated_x * 0.075;
    float screen_y = centered.y * 0.13 - rotated_z * 0.045;
    float depth = 0.5 + rotated_z * 0.001 - centered.y * 0.001;
    gl_Position = vec4(screen_x, screen_y, depth, 1.0);
    v_uv = uv;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 140
in vec2 v_uv;
out vec4 out_color;

void main() {
    float texture_hint = 0.94 + fract(v_uv.x * 16.0) * 0.06;
    vec3 grass = vec3(0.30, 0.58, 0.25);
    out_color = vec4(grass * texture_hint, 1.0);
}
"#;

pub struct OpenGlRenderer {
    display: glium::backend::glutin::Display<WindowSurface>,
    chunk_vertices: VertexBuffer<Vertex>,
    program: Program,
    started: Instant,
}

impl OpenGlRenderer {
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

        let chunk_vertices = VertexBuffer::new(&display, &demo_chunk_vertices())
            .map_err(|error| RendererError::new(format!("Chunk vertex buffer: {error}")))?;
        let program = Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None)
            .map_err(|error| RendererError::new(format!("Chunk shader program: {error}")))?;

        Ok((
            window,
            Self {
                display,
                chunk_vertices,
                program,
                started: Instant::now(),
            },
        ))
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
        let mut target = self.display.draw();
        let clear = frame.clear_color;
        target.clear_color_and_depth(
            (clear.red, clear.green, clear.blue, clear.alpha),
            1.0,
        );

        let parameters = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Depth::default()
            },
            ..DrawParameters::default()
        };

        let elapsed = self.started.elapsed().as_secs_f32();
        target
            .draw(
                &self.chunk_vertices,
                NoIndices(glium::index::PrimitiveType::TrianglesList),
                &self.program,
                &uniform! { u_time: elapsed },
                &parameters,
            )
            .map_err(|error| RendererError::new(format!("OpenGL chunk draw: {error}")))?;

        target
            .finish()
            .map_err(|error| RendererError::new(format!("OpenGL present: {error}")))
    }
}

/// Builds a small test terrain through the same CPU mesher used by world chunks.
fn demo_chunk_vertices() -> Vec<Vertex> {
    let mut registry = BlockRegistry::new();
    registry.register(BlockDefinition {
        id: BlockId(1),
        name: "book:grass_block".to_owned(),
        atlas_tile: AtlasTile { x: 0, y: 0 },
        solid: true,
        opaque: true,
    });

    let mut section = ChunkSection::new();
    for z in 0..SECTION_EDGE {
        for x in 0..SECTION_EDGE {
            section.set(x, 0, z, BlockId(1));
        }
    }

    let mesh = mesh_section(&section, &registry, 16, 16);
    mesh.vertices
        .into_iter()
        .map(|vertex| Vertex {
            position: vertex.position,
            uv: vertex.uv,
        })
        .collect()
}
