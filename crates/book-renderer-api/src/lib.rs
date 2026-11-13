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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderFrame {
    pub clear_color: ClearColor,
}

impl Default for RenderFrame {
    fn default() -> Self { Self { clear_color: ClearColor::SKY } }
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
