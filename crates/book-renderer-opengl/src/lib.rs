use book_renderer_api::{GraphicsApi, RendererBackend, RendererError, RendererInfo, RenderFrame};
use glium::backend::glutin::SimpleWindowBuilder;
use glium::index::NoIndices;
use glium::uniform;
use glium::{implement_vertex, Depth, DepthTest, DrawParameters, Program, Surface, VertexBuffer};
use glutin::surface::WindowSurface;
use std::time::Instant;

#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
implement_vertex!(Vertex, position, color);

const VERTEX_SHADER: &str = r#"
#version 140
in vec3 position;
in vec3 color;
out vec3 v_color;
uniform float u_time;
void main() {
    float c = cos(u_time * 0.5);
    float s = sin(u_time * 0.5);
    vec3 p = vec3(position.x * c + position.z * s,
                  position.y,
                  -position.x * s + position.z * c);
    gl_Position = vec4(p.xy * 0.65, p.z * 0.2, 1.0);
    v_color = color;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 140
in vec3 v_color;
out vec4 out_color;
void main() { out_color = vec4(v_color, 1.0); }
"#;

pub struct OpenGlRenderer {
    display: glium::backend::glutin::Display<WindowSurface>,
    vertices: VertexBuffer<Vertex>,
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

        let vertices = VertexBuffer::new(&display, &cube_vertices())
            .map_err(|e| RendererError::new(format!("Vertex buffer: {e}")))?;
        let program = Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None)
            .map_err(|e| RendererError::new(format!("Shader program: {e}")))?;

        Ok((window, Self {
            display,
            vertices,
            program,
            started: Instant::now(),
        }))
    }
}

impl RendererBackend for OpenGlRenderer {
    fn info(&self) -> RendererInfo {
        RendererInfo {
            name: "Book OpenGL Renderer".to_owned(),
            api: GraphicsApi::OpenGl,
            api_version: "OpenGL via Glium/Glutin".to_owned(),
        }
    }

    fn render(&mut self, frame: &RenderFrame) -> Result<(), RendererError> {
        let mut target = self.display.draw();
        let c = frame.clear_color;
        target.clear_color_and_depth((c.red, c.green, c.blue, c.alpha), 1.0);

        let params = DrawParameters {
            depth: Depth { test: DepthTest::IfLess, write: true, ..Depth::default() },
            ..DrawParameters::default()
        };
        let elapsed = self.started.elapsed().as_secs_f32();
        target.draw(
            &self.vertices,
            NoIndices(glium::index::PrimitiveType::TrianglesList),
            &self.program,
            &uniform! { u_time: elapsed },
            &params,
        ).map_err(|e| RendererError::new(format!("OpenGL draw: {e}")))?;

        target.finish().map_err(|e| RendererError::new(format!("OpenGL present: {e}")))
    }
}

fn cube_vertices() -> Vec<Vertex> {
    let p = [
        [-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5, 0.5, -0.5],
        [-0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [-0.5, 0.5, -0.5],
        [-0.5, -0.5, 0.5], [0.5, -0.5, 0.5], [0.5, 0.5, 0.5],
        [-0.5, -0.5, 0.5], [0.5, 0.5, 0.5], [-0.5, 0.5, 0.5],
        [-0.5, -0.5, -0.5], [-0.5, -0.5, 0.5], [-0.5, 0.5, 0.5],
        [-0.5, -0.5, -0.5], [-0.5, 0.5, 0.5], [-0.5, 0.5, -0.5],
        [0.5, -0.5, -0.5], [0.5, 0.5, 0.5], [0.5, -0.5, 0.5],
        [0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [0.5, 0.5, 0.5],
        [-0.5, -0.5, -0.5], [0.5, -0.5, 0.5], [0.5, -0.5, -0.5],
        [-0.5, -0.5, -0.5], [-0.5, -0.5, 0.5], [0.5, -0.5, 0.5],
        [-0.5, 0.5, -0.5], [0.5, 0.5, -0.5], [0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5], [0.5, 0.5, 0.5], [-0.5, 0.5, 0.5],
    ];
    p.iter().enumerate().map(|(i, position)| {
        let shade = 0.35 + (i / 6) as f32 * 0.09;
        Vertex { position: *position, color: [shade, shade + 0.12, 0.95] }
    }).collect()
}
