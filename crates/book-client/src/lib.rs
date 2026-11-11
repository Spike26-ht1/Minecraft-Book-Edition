#![cfg(target_os = "android")]

use book_renderer_api::{ClearColor, RendererBackend, RenderFrame};
use book_renderer_opengl::OpenGlRenderer;
use glium::winit::event::{ElementState, Event, TouchPhase, WindowEvent};
use glium::winit::keyboard::{KeyCode, PhysicalKey};
use glium::winit::platform::android::activity::AndroidApp;
use glium::winit::platform::android::EventLoopBuilderExtAndroid;

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub fn android_main(app: AndroidApp) {
    let mut builder = glium::winit::event_loop::EventLoopBuilder::new();
    builder.with_android_app(app);
    let event_loop = builder
        .build()
        .expect("failed to create Android event loop");

    let frame = RenderFrame {
        clear_color: ClearColor::SKY,
    };
    let mut renderer_state = None;

    let _ = event_loop.run(move |event, active_loop| match event {
        Event::Resumed => {
            if renderer_state.is_none() {
                match OpenGlRenderer::new(
                    active_loop,
                    "Minecraft Book Edition 26.60",
                    1280,
                    720,
                ) {
                    Ok((window, renderer)) => {
                        renderer_state = Some((window, renderer));
                    }
                    Err(error) => {
                        eprintln!("Failed to initialize Android OpenGL ES renderer: {error}");
                        active_loop.exit();
                    }
                }
            }
        }
        Event::Suspended => {
            renderer_state = None;
        }
        Event::WindowEvent { window_id, event } => {
            if let Some((window, renderer)) = renderer_state.as_mut() {
                if window.id() != window_id {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => active_loop.exit(),
                    WindowEvent::KeyboardInput { event, .. }
                        if event.state == ElementState::Pressed
                            && event.physical_key == PhysicalKey::Code(KeyCode::F3) =>
                    {
                        renderer.toggle_debug();
                        window.request_redraw();
                    }
                    WindowEvent::Touch(touch)
                        if touch.phase == TouchPhase::Started =>
                    {
                        let size = window.inner_size();
                        let touch_x = touch.location.x;
                        let touch_y = touch.location.y;
                        if touch_x >= size.width.saturating_sub(96) as f64 && touch_y <= 64.0 {
                            renderer.toggle_debug();
                            window.request_redraw();
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        if let Err(error) = renderer.render(&frame) {
                            eprintln!("Android renderer error: {error}");
                            active_loop.exit();
                        }
                    }
                    _ => {}
                }
            }
        }
        Event::AboutToWait => {
            if let Some((window, _)) = renderer_state.as_ref() {
                window.request_redraw();
            }
        }
        _ => {}
    });
}
