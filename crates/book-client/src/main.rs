#[cfg(not(all(target_arch = "x86_64", target_pointer_width = "64")))]
compile_error!("Minecraft Book Edition 26.60 test client currently supports x86_64 64-bit targets only.");

use book_renderer_api::{ClearColor, RendererBackend, RenderFrame};
use book_renderer_opengl::OpenGlRenderer;
use glium::winit::event::{ElementState, Event, WindowEvent};
use glium::winit::keyboard::{KeyCode, PhysicalKey};

fn main() {
    let event_loop = glium::winit::event_loop::EventLoopBuilder::new()
        .build()
        .expect("failed to create window event loop");
    let (window, mut renderer) =
        OpenGlRenderer::new(&event_loop, "Minecraft Book Edition 26.60", 1280, 720)
            .unwrap_or_else(|error| {
                eprintln!("Failed to initialize Book Edition renderer: {error}");
                std::process::exit(1);
            });

    let info = renderer.info();
    println!("{} — {:?} ({})", info.name, info.api, info.api_version);
    println!("Target: x86_64 (64-bit)");
    println!("Press F3 to toggle the debug overlay.");

    let frame = RenderFrame { clear_color: ClearColor::SKY };
    let _ = event_loop.run(move |event, window_target| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => window_target.exit(),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && event.physical_key == PhysicalKey::Code(KeyCode::F3) =>
            {
                renderer.toggle_debug();
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = renderer.render(&frame) {
                    eprintln!("Renderer error: {error}");
                    window_target.exit();
                }
            }
            _ => {}
        },
        Event::AboutToWait => window.request_redraw(),
        _ => {}
    });
}
