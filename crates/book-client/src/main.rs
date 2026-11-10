use book_renderer_api::{ClearColor, RendererBackend, RenderFrame};
use book_renderer_opengl::OpenGlRenderer;

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

    let frame = RenderFrame {
        clear_color: ClearColor::SKY,
    };

    let _ = event_loop.run(move |event, window_target| {
        match event {
            glium::winit::event::Event::WindowEvent { event, .. } => match event {
                glium::winit::event::WindowEvent::CloseRequested => window_target.exit(),
                glium::winit::event::WindowEvent::RedrawRequested => {
                    if let Err(error) = renderer.render(&frame) {
                        eprintln!("Renderer error: {error}");
                        window_target.exit();
                    }
                }
                _ => {}
            },
            glium::winit::event::Event::AboutToWait => window.request_redraw(),
            _ => {}
        }
    });
}
