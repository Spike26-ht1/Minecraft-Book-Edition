//! First Book64 backend: OpenGL ES 3.0 voxel renderer.
//!
//! This milestone draws a deterministic test chunk with grass, dirt and stone.
//! It is a renderer proof, not the final terrain or block system.

use std::ffi::{c_void, CStr, CString};
use std::mem::size_of;
use std::ptr;
use std::time::Instant;

use book_meshing::build_chunk_mesh;
use book_world::Chunk;
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

/// Owns the GPU resources for the Book64 voxel test scene.
///
/// All methods must be called on the thread where the current EGL context is
/// bound. The Android host guarantees this by calling them from GLSurfaceView.
pub struct GlesRenderer {
    program: u32,
    vertex_array: u32,
    vertex_buffer: u32,
    mvp_location: i32,
    vertex_count: i32,
    started_at: Instant,
    width: i32,
    height: i32,
    camera_position: Vec3,
    camera_yaw: f32,
    camera_pitch: f32,
    move_forward: f32,
    move_strafe: f32,
    move_vertical: f32,
    last_frame: Instant,
}

impl GlesRenderer {
    /// Loads GLES entry points and uploads the first generated voxel chunk.
    ///
    /// # Safety
    /// A valid OpenGL ES 3.0 context must be current on the calling thread.
    pub unsafe fn initialize(
        loader: impl FnMut(&'static str) -> *const c_void,
    ) -> Result<Self, String> {
        gl::load_with(loader);

        if !gl::CreateShader::is_loaded() || !gl::GenVertexArrays::is_loaded() {
            return Err("OpenGL ES 3.0 entry points are unavailable".to_owned());
        }

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

        let chunk = Chunk::generate_test_terrain();
        let mesh = build_chunk_mesh(&chunk);
        if mesh.is_empty() {
            gl::DeleteProgram(program);
            return Err("The test chunk produced an empty mesh".to_owned());
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
            started_at: Instant::now(),
            width: 1,
            height: 1,
            camera_position: Vec3::new(22.0, 18.0, 25.0),
            camera_yaw: -0.576,
            camera_pitch: -0.49,
            move_forward: 0.0,
            move_strafe: 0.0,
            move_vertical: 0.0,
            last_frame: Instant::now(),
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

    /// Renders one animated voxel terrain frame.
    ///
    /// # Safety
    /// A valid OpenGL ES context must be current on the calling thread.
    pub unsafe fn draw(&mut self) {
        let now = Instant::now();
        let delta_seconds = (now - self.last_frame).as_secs_f32().clamp(0.0, 0.1);
        self.last_frame = now;
        let forward = self.camera_front();
        let right = forward.cross(Vec3::Y).normalize_or_zero();
        let movement = forward * self.move_forward
            + right * self.move_strafe
            + Vec3::Y * self.move_vertical;
        self.camera_position += movement * (6.0 * delta_seconds);

        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::UseProgram(self.program);

        let aspect = self.width as f32 / self.height as f32;
        let projection = Mat4::perspective_rh_gl(48.0_f32.to_radians(), aspect, 0.1, 100.0);
        let front = self.camera_front();
        let view = Mat4::look_at_rh(
            self.camera_position,
            self.camera_position + front,
            Vec3::Y,
        );
        let angle = self.started_at.elapsed().as_secs_f32() * 0.12;
        let model = Mat4::from_translation(Vec3::new(8.0, 3.0, 8.0))
            * Mat4::from_rotation_y(angle)
            * Mat4::from_translation(Vec3::new(-8.0, -3.0, -8.0));
        let mvp = projection * view * model;

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

    pub fn set_movement(&mut self, forward: f32, strafe: f32, vertical: f32) {
        self.move_forward = forward.clamp(-1.0, 1.0);
        self.move_strafe = strafe.clamp(-1.0, 1.0);
        self.move_vertical = vertical.clamp(-1.0, 1.0);
    }

    pub fn rotate_camera(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.camera_yaw += yaw_delta;
        self.camera_pitch = (self.camera_pitch + pitch_delta).clamp(-1.45, 1.45);
    }

    pub fn debug_info(&self) -> String {
        format!(
            "XYZ: {:.2}, {:.2}, {:.2}\nYaw/Pitch: {:.2} / {:.2}\nViewport: {} × {}\nVertices: {}",
            self.camera_position.x,
            self.camera_position.y,
            self.camera_position.z,
            self.camera_yaw.to_degrees(),
            self.camera_pitch.to_degrees(),
            self.width,
            self.height,
            self.vertex_count
        )
    }

    fn camera_front(&self) -> Vec3 {
        Vec3::new(
            self.camera_yaw.cos() * self.camera_pitch.cos(),
            self.camera_pitch.sin(),
            self.camera_yaw.sin() * self.camera_pitch.cos(),
        )
        .normalize_or_zero()
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

    let mut buffer = vec![0_i8; length as usize];
    gl::GetShaderInfoLog(shader, length, ptr::null_mut(), buffer.as_mut_ptr());
    CStr::from_ptr(buffer.as_ptr()).to_string_lossy().into_owned()
}

unsafe fn program_log(program: u32) -> String {
    let mut length = 0;
    gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut length);
    if length <= 1 {
        return "no program log was provided".to_owned();
    }

    let mut buffer = vec![0_i8; length as usize];
    gl::GetProgramInfoLog(program, length, ptr::null_mut(), buffer.as_mut_ptr());
    CStr::from_ptr(buffer.as_ptr()).to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_test_chunk_has_visible_voxel_geometry() {
        let chunk = Chunk::generate_test_terrain();
        let mesh = build_chunk_mesh(&chunk);

        assert!(!mesh.is_empty());
        assert_eq!(mesh.vertex_count() % 3, 0);
    }
}
