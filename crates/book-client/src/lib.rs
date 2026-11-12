#![cfg(target_os = "android")]

use book_renderer_api::{ClearColor, RenderFrame, RendererBackend};
use book_renderer_opengl::OpenGlRenderer;
use glium::winit::event::{ElementState, Event, TouchPhase, WindowEvent};
use glium::winit::keyboard::{KeyCode, PhysicalKey};
use glium::winit::platform::android::activity::AndroidApp;
use glium::winit::platform::android::EventLoopBuilderExtAndroid;

fn initialize_android_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("BookEdition")
            .with_max_level(log::LevelFilter::Trace),
    );
    log::set_max_level(log::LevelFilter::Trace);

    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("PANIC during Android startup: {panic_info}");
    }));
    log::info!("Minecraft Book Edition Pre-Alpha 1.1.0: android_main entered");
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub fn android_main(app: AndroidApp) {
    initialize_android_logging();

    let mut builder = glium::winit::event_loop::EventLoopBuilder::new();
    builder.with_android_app(app);

    let event_loop = match builder.build() {
        Ok(event_loop) => {
            log::info!("Winit Android event loop created");
            event_loop
        }
        Err(error) => {
            log::error!("Failed to create Winit Android event loop: {error}");
            return;
        }
    };

    let frame = RenderFrame {
        clear_color: ClearColor::SKY,
    };
    let mut renderer_state = None;

    let result = event_loop.run(move |event, active_loop| match event {
        Event::Resumed => {
            log::info!("Android lifecycle: Resumed");
            if renderer_state.is_none() {
                log::info!("Starting OpenGL ES 3 renderer initialization");
                match OpenGlRenderer::new(
                    active_loop,
                    "Minecraft Book Edition — Pre-Alpha 1.1.0",
                    1280,
                    720,
                ) {
                    Ok((window, renderer)) => {
                        log::info!("OpenGL ES renderer initialized successfully");
                        renderer_state = Some((window, renderer));
                    }
                    Err(error) => {
                        log::error!("OpenGL ES renderer initialization failed: {error}");
                        active_loop.exit();
                    }
                }
            }
        }
        Event::Suspended => {
            log::info!("Android lifecycle: Suspended; releasing renderer");
            renderer_state = None;
        }
        Event::WindowEvent { window_id, event } => {
            if let Some((window, renderer)) = renderer_state.as_mut() {
                if window.id() != window_id {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => {
                        log::info!("Android window requested close");
                        active_loop.exit();
                    }
                    WindowEvent::KeyboardInput { event, .. }
                        if event.state == ElementState::Pressed
                            && event.physical_key == PhysicalKey::Code(KeyCode::F3) =>
                    {
                        renderer.toggle_debug();
                        window.request_redraw();
                    }
                    WindowEvent::Touch(touch) if touch.phase == TouchPhase::Started => {
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
                            log::error!("Android renderer frame failed: {error}");
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
        Event::LoopExiting => {
            log::info!("Winit Android event loop is exiting");
        }
        _ => {}
    });

    if let Err(error) = result {
        log::error!("Winit Android event loop returned an error: {error}");
    } else {
        log::info!("Winit Android event loop returned normally");
    }
}
