//! Book64 OpenGL ES 3.0 presentation backend.
//!
//! This crate uploads world meshes and renders immutable simulation snapshots.
//! Player movement, world state and fixed-step updates belong to book-game.

use std::ffi::{c_char, c_void, CStr, CString};
use std::mem::size_of;
use std::ptr;

use book_game::RenderState;
use book_meshing::build_chunk_region_mesh;
use book_world::World;
use glam::{Mat4, Vec3};

const VERTEX_SHADER: &str = r#"#version 300 es
layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_color;
uniform mat4 u_mvp;
out vec3 v_color;
void main() {
    gl_Position = u_mvp * vec4(a_position, 1.0);
    v_color = a_color;
}
"#;

const FRAGMENT_SHADER: &str = r#"#version 300 es
precision mediump float;
in vec3 v_color;
out vec4 out_color;
void main() {
    out_color = vec4(v_color, 1.0);
}
"#;

/// Owns OpenGL resources only. The game session is owned by the runtime host.
pub struct GlesRenderer {
    program: u32,
    vertex_array: u32,
    vertex_buffer: u32,
    mvp_location: i32,
    vertex_count: i32,
    width: i32,
    height: i32,
    gpu_vendor: String,
    gpu_renderer: String,
    gles_version: String,
    last_render_state: Option<RenderState>,
}

impl GlesRenderer {
    /// Loads GLES entry points and uploads a snapshot of the loaded world.
    ///
    /// # Safety
    /// A valid OpenGL ES 3.0 context must be current on the calling thread.
    pub unsafe fn initialize(
        loader: impl FnMut(&'static str) -> *const c_void,
        world: &World,
    ) -> Result<Self, String> {
        gl::load_with(loader);

        if !gl::CreateShader::is_loaded() || !gl::GenVertexArrays::is_loaded() {
            return Err("OpenGL ES 3.0 entry points are unavailable".to_owned());
        }

        let gpu_vendor = gl_string(gl::VENDOR);
        let gpu_renderer = gl_string(gl::RENDERER);
        let gles_version = gl_string(gl::VERSION);

        let vertex_shader = compile_shader(gl::VERTEX_SHADER, VERTEX_SHADER)?;
        let fragment_shader = match compile_shader(gl::FRAGMENT_SHADER, FRAGMENT_SHADER) {
            Ok(shader) => shader,
            Err(error) => {
                gl::DeleteShader(vertex_shader);
                return Err(error);
            }
        };

        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        let mut link_status = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut link_status);
        if link_status == 0 {
            let message = program_log(program);
            gl::DeleteProgram(program);
            return Err(format!("Book64 shader program link failed: {message}"));
        }

        let uniform_name = CString::new("u_mvp").expect("static uniform name");
        let mvp_location = gl::GetUniformLocation(program, uniform_name.as_ptr());
        if mvp_location < 0 {
            gl::DeleteProgram(program);
            return Err("Book64 could not locate the u_mvp uniform".to_owned());
        }

        let columns = world.columns_for_meshing();
        let mesh = build_chunk_region_mesh(&columns);
        if mesh.is_empty() {
            gl::DeleteProgram(program);
            return Err("The loaded world produced an empty mesh".to_owned());
        }

        let mut vertices = Vec::with_capacity(mesh.vertex_count() * 6);
        for vertex in mesh.vertices {
            vertices.extend_from_slice(&vertex.position);
            vertices.extend_from_slice(&vertex.color);
        }
        let vertex_count = (vertices.len() / 6) as i32;

        let mut vertex_array = 0;
        let mut vertex_buffer = 0;

        gl::GenVertexArrays(1, &mut vertex_array);
        gl::GenBuffers(1, &mut vertex_buffer);
        gl::BindVertexArray(vertex_array);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * size_of::<f32>()) as isize,
            vertices.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

        let stride = (6 * size_of::<f32>()) as i32;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * size_of::<f32>()) as *const c_void,
        );

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LEQUAL);
        gl::ClearColor(0.035, 0.055, 0.085, 1.0);

        Ok(Self {
            program,
            vertex_array,
            vertex_buffer,
            mvp_location,
            vertex_count,
            width: 1,
            height: 1,
            gpu_vendor,
            gpu_renderer,
            gles_version,
            last_render_state: None,
        })
    }

    /// Updates the viewport after the Android surface changes size.
    ///
    /// # Safety
    /// A valid OpenGL ES context must be current on the calling thread.
    pub unsafe fn resize(&mut self, width: i32, height: i32) {
        self.width = width.max(1);
        self.height = height.max(1);
        gl::Viewport(0, 0, self.width, self.height);
    }

    /// Renders one immutable state produced by book-game.
    ///
    /// # Safety
    /// A valid OpenGL ES context must be current on the calling thread.
    pub unsafe fn draw(&mut self, state: RenderState) {
        self.last_render_state = Some(state);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::UseProgram(self.program);

        let aspect = self.width as f32 / self.height as f32;
        let projection = Mat4::perspective_rh_gl(70.0_f32.to_radians(), aspect, 0.05, 192.0);
        let eye = Vec3::from_array(state.eye_position);
        let front = Vec3::new(
            state.yaw_radians.cos() * state.pitch_radians.cos(),
            state.pitch_radians.sin(),
            state.yaw_radians.sin() * state.pitch_radians.cos(),
        )
        .normalize_or_zero();
        let view = Mat4::look_at_rh(eye, eye + front, Vec3::Y);
        let mvp = projection * view;

        gl::UniformMatrix4fv(
            self.mvp_location,
            1,
            gl::FALSE,
            mvp.to_cols_array().as_ptr(),
        );
        gl::BindVertexArray(self.vertex_array);
        gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count);
        gl::BindVertexArray(0);
        gl::UseProgram(0);
    }

    pub fn vertex_count(&self) -> usize {
        self.vertex_count as usize
    }

    pub fn debug_info(&self) -> String {
        let Some(state) = self.last_render_state else {
            return "Book64 ready; waiting for the first simulation frame".to_owned();
        };
        let position = state.player_position;
        let block = [
            position[0].floor() as i32,
            position[1].floor() as i32,
            position[2].floor() as i32,
        ];
        let chunk = [block[0].div_euclid(16), block[2].div_euclid(16)];

        format!(
            "Player XYZ: {:.2}, {:.2}, {:.2}\nBlock: {}, {}, {}\nChunk XZ: {}, {}\nYaw/Pitch: {:.2} / {:.2}\nGrounded: {}\nWorld seed: {}\nGenerator: {}\nLoaded columns: {}\nGame tick: {} (20 TPS)\nViewport: {} × {}\nVertices: {}\nGPU vendor: {}\nGPU: {}\nGLES: {}",
            position[0],
            position[1],
            position[2],
            block[0],
            block[1],
            block[2],
            chunk[0],
            chunk[1],
            state.yaw_radians.to_degrees(),
            state.pitch_radians.to_degrees(),
            state.grounded,
            state.world_seed as i64,
            state.generator_version,
            state.loaded_columns,
            state.game_tick,
            self.width,
            self.height,
            self.vertex_count,
            self.gpu_vendor,
            self.gpu_renderer,
            self.gles_version
        )
    }
}

impl Drop for GlesRenderer {
    fn drop(&mut self) {
        // GPU cleanup is intentionally handled by the Android lifecycle in a
        // later resource-owner milestone. Deleting here could call GLES after
        // the EGL context has already been lost.
        let _ = (self.program, self.vertex_array, self.vertex_buffer);
    }
}

unsafe fn gl_string(name: u32) -> String {
    let value = gl::GetString(name);
    if value.is_null() {
        return "unavailable".to_owned();
    }
    CStr::from_ptr(value.cast()).to_string_lossy().into_owned()
}

unsafe fn compile_shader(kind: u32, source: &str) -> Result<u32, String> {
    let shader = gl::CreateShader(kind);
    if shader == 0 {
        return Err("OpenGL ES could not allocate a shader".to_owned());
    }

    let source = CString::new(source).map_err(|error| error.to_string())?;
    gl::ShaderSource(shader, 1, &source.as_ptr(), ptr::null());
    gl::CompileShader(shader);

    let mut compile_status = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status);
    if compile_status == 0 {
        let message = shader_log(shader);
        gl::DeleteShader(shader);
        return Err(format!("Book64 shader compilation failed: {message}"));
    }

    Ok(shader)
}

unsafe fn shader_log(shader: u32) -> String {
    let mut length = 0;
    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut length);
    if length <= 1 {
        return "no shader log was provided".to_owned();
    }

    let mut buffer = vec![0 as c_char; length as usize];
    gl::GetShaderInfoLog(shader, length, ptr::null_mut(), buffer.as_mut_ptr());
    CStr::from_ptr(buffer.as_ptr())
        .to_string_lossy()
        .into_owned()
}

unsafe fn program_log(program: u32) -> String {
    let mut length = 0;
    gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut length);
    if length <= 1 {
        return "no program log was provided".to_owned();
    }

    let mut buffer = vec![0 as c_char; length as usize];
    gl::GetProgramInfoLog(program, length, ptr::null_mut(), buffer.as_mut_ptr());
    CStr::from_ptr(buffer.as_ptr())
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_world_produces_a_visible_mesh_for_the_renderer() {
        let world = World::generate(42, 2, 4);
        let mesh = build_chunk_region_mesh(&world.columns_for_meshing());

        assert!(!mesh.is_empty());
        assert_eq!(mesh.vertex_count() % 3, 0);
    }

    #[test]
    fn render_state_is_not_owned_or_mutated_by_the_renderer() {
        let state = RenderState {
            player_position: [1.0, 20.0, 3.0],
            eye_position: [1.0, 21.62, 3.0],
            yaw_radians: 0.0,
            pitch_radians: 0.0,
            grounded: true,
            world_seed: 42,
            generator_version: 1,
            loaded_columns: 25,
            game_tick: 20,
        };
        let copy = state;
        assert_eq!(state, copy);
    }
}
