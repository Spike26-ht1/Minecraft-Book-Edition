#![cfg(target_os = "android")]

use book_renderer_api::{ClearColor, RenderFrame, RendererBackend};
use book_renderer_opengl::OpenGlRenderer;
use glium::winit::event::{ElementState, Event, TouchPhase, WindowEvent};
use glium::winit::event_loop::ControlFlow;
use glium::winit::keyboard::{KeyCode, PhysicalKey};
use glium::winit::platform::android::activity::AndroidApp;
use glium::winit::platform::android::EventLoopBuilderExtAndroid;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const PACKAGE_ID: &str = "com.spike26.minecraftbookedition";
const FRAME_INTERVAL: Duration = Duration::from_nanos(16_666_667);

fn initialize_android_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("BookEdition")
            .with_max_level(log::LevelFilter::Trace),
    );
    log::set_max_level(log::LevelFilter::Trace);
    std::panic::set_hook(Box::new(|panic_info| {
        let message = format!("PANIC during Android startup: {panic_info}");
        log::error!("{message}");
        book_renderer_opengl::record_android_diagnostic(&message);
    }));
    log::info!(
        "Minecraft Book Edition Pre-Alpha 1.2.0: android_main entered; process_id={}",
        std::process::id()
    );
    book_renderer_opengl::record_android_diagnostic(&format!(
        "android_main entered; process_id={}",
        std::process::id()
    ));
}

fn derive_media_root(external_files_path: &Path) -> Option<PathBuf> {
    // AndroidApp exposes Android/data/<package>/files. Its app-specific media
    // directory is the sibling Android/media/<package>.
    let package_dir = external_files_path.parent()?;
    if package_dir.file_name()?.to_str()? != PACKAGE_ID {
        return None;
    }
    let data_dir = package_dir.parent()?;
    if data_dir.file_name()?.to_str()? != "data" {
        return None;
    }
    let android_dir = data_dir.parent()?;
    if android_dir.file_name()?.to_str()? != "Android" {
        return None;
    }
    Some(android_dir.join("media").join(PACKAGE_ID))
}

fn append_startup_log(path: &Path, message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    match OpenOptions::new().create(true).append(true).open(path) {
        Ok(mut file) => {
            if let Err(error) = writeln!(file, "[{timestamp}] {message}") {
                log::warn!("Could not write startup log {}: {error}", path.display());
            }
        }
        Err(error) => log::warn!("Could not open startup log {}: {error}", path.display()),
    }
}

fn initialize_storage(app: &AndroidApp) -> Option<PathBuf> {
    let external_files_path = app.external_data_path()?;
    let root = derive_media_root(&external_files_path)?;
    let directories = [
        "game/minecraftWorlds",
        "game/resource_packs",
        "game/behavior_packs",
        "game/development_resource_packs",
        "game/development_behavior_packs",
        "game/minecraftpe",
        "game/logs",
        "launcher/config",
        "launcher/profiles",
        "launcher/instances",
        "launcher/downloads",
        "launcher/cache",
        "launcher/logs",
    ];
    for relative in directories {
        let directory = root.join(relative);
        if let Err(error) = fs::create_dir_all(&directory) {
            log::error!("Could not create directory {}: {error}", directory.display());
            return None;
        }
    }
    let game_log_path = root.join("game/logs/latest.log");
    let launcher_log_path = root.join("launcher/logs/launcher.log");
    book_renderer_opengl::set_android_diagnostic_log_paths(
        game_log_path.clone(),
        launcher_log_path.clone(),
    );
    append_startup_log(
        &game_log_path,
        "Minecraft Book Edition entered; game storage initialized",
    );
    append_startup_log(
        &launcher_log_path,
        "Minecraft Book Edition launcher entered; launcher storage initialized",
    );
    book_renderer_opengl::record_android_diagnostic("Storage initialization completed");
    log::info!("Android media storage initialized at {}", root.display());
    Some(root)
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub fn android_main(app: AndroidApp) {
    initialize_android_logging();
    let _storage_root = match initialize_storage(&app) {
        Some(root) => {
            book_renderer_opengl::record_android_diagnostic("android_main: storage is ready");
            Some(root)
        }
        None => {
            log::warn!("Android/media storage could not be initialized; continuing without it");
            book_renderer_opengl::record_android_diagnostic(
                "android_main: storage initialization failed; continuing without media storage",
            );
            None
        }
    };

    book_renderer_opengl::record_android_diagnostic("Winit: creating Android event loop");
    let mut builder = glium::winit::event_loop::EventLoopBuilder::new();
    builder.with_android_app(app);
    let event_loop = match builder.build() {
        Ok(event_loop) => {
            log::info!("Winit Android event loop created");
            book_renderer_opengl::record_android_diagnostic("Winit: Android event loop created");
            event_loop
        }
        Err(error) => {
            log::error!("Failed to create Winit Android event loop: {error}");
            book_renderer_opengl::record_android_diagnostic(
                &format!("Winit ERROR: could not create Android event loop: {error}"),
            );
            return;
        }
    };

    let mut frame = RenderFrame::default();
    let mut renderer_state = None;
    let mut pressed_keys = HashSet::new();
    let mut active_touches: HashMap<u64, (f64, f64)> = HashMap::new();
    let mut next_frame_deadline: Option<Instant> = None;
    let result = event_loop.run(move |event, active_loop| match event {
        Event::Resumed => {
            log::info!("Android lifecycle: Resumed");
            book_renderer_opengl::record_android_diagnostic("Android lifecycle: Resumed");
            if renderer_state.is_none() {
                log::info!("Starting OpenGL ES 3 renderer initialization");
                book_renderer_opengl::record_android_diagnostic(
                    "Renderer: starting OpenGL ES 3 initialization",
                );
                match OpenGlRenderer::new(
                    active_loop,
                    "Minecraft Book Edition — Pre-Alpha 1.2.0",
                    1280,
                    720,
                ) {
                    Ok((window, renderer)) => {
                        log::info!("OpenGL ES renderer initialized successfully");
                        book_renderer_opengl::record_android_diagnostic(
                            "Renderer: OpenGL ES renderer initialized successfully",
                        );
                        renderer_state = Some((window, renderer));
                    }
                    Err(error) => {
                        log::error!("OpenGL ES renderer initialization failed: {error}");
                        book_renderer_opengl::record_android_diagnostic(
                            &format!(
                                "Renderer ERROR: initialization failed; keeping Android event loop alive and waiting for the next resume: {error}"
                            ),
                        );
                    }
                }
            }
            if let Some((window, _)) = renderer_state.as_ref() {
                window.request_redraw();
                let deadline = Instant::now() + FRAME_INTERVAL;
                next_frame_deadline = Some(deadline);
                active_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            } else {
                next_frame_deadline = None;
                active_loop.set_control_flow(ControlFlow::Wait);
            }
        }
        Event::Suspended => {
            log::info!("Android lifecycle: Suspended; releasing renderer");
            book_renderer_opengl::record_android_diagnostic(
                "Android lifecycle: Suspended; releasing renderer",
            );
            renderer_state = None;
            next_frame_deadline = None;
            active_loop.set_control_flow(ControlFlow::Wait);
            book_renderer_opengl::record_android_diagnostic(&format!(
                "Android lifecycle: renderer released; process_id={}",
                std::process::id()
            ));
        }
        Event::WindowEvent { window_id, event } => {
            let window_matches = renderer_state
                .as_ref()
                .map(|(window, _)| window.id() == window_id)
                .unwrap_or(false);
            if !window_matches { return; }

            match event {
                WindowEvent::Focused(focused) => {
                    log::info!("Android window focus changed: focused={focused}");
                    book_renderer_opengl::record_android_diagnostic(&format!(
                        "Android window focus changed: focused={focused}"
                    ));
                }
                WindowEvent::CloseRequested => {
                    log::info!("Android window requested close; releasing renderer");
                    book_renderer_opengl::record_android_diagnostic(
                        "Android window close requested; renderer released without ending the event loop",
                    );
                    renderer_state = None;
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        if event.state == ElementState::Pressed {
                            pressed_keys.insert(key);
                            if key == KeyCode::F3 {
                                if let Some((window, renderer)) = renderer_state.as_mut() {
                                    renderer.toggle_debug();
                                    window.request_redraw();
                                }
                            }
                        } else {
                            pressed_keys.remove(&key);
                        }
                    }
                }
                WindowEvent::Touch(touch) => {
                    let touch_id = touch.id;
                    let location = (touch.location.x, touch.location.y);
                    match touch.phase {
                        TouchPhase::Started => {
                            active_touches.insert(touch_id, location);
                            if let Some((window, renderer)) = renderer_state.as_mut() {
                                let size = window.inner_size();
                                if location.0 >= size.width.saturating_sub(96) as f64
                                    && location.1 <= 64.0
                                {
                                    renderer.toggle_debug();
                                    window.request_redraw();
                                }
                            }
                        }
                        TouchPhase::Moved => {
                            if let Some(previous) = active_touches.insert(touch_id, location) {
                                frame.camera.look(
                                    ((location.0 - previous.0) as f32) * 0.004,
                                    -((location.1 - previous.1) as f32) * 0.004,
                                );
                                if let Some((window, _)) = renderer_state.as_ref() {
                                    window.request_redraw();
                                }
                            }
                        }
                        TouchPhase::Ended | TouchPhase::Cancelled => {
                            active_touches.remove(&touch_id);
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    let render_result = renderer_state
                        .as_mut()
                        .map(|(_, renderer)| renderer.render(&frame));
                    if let Some(Err(error)) = render_result {
                        log::error!("Android renderer frame failed: {error}");
                        book_renderer_opengl::record_android_diagnostic(
                            &format!("Renderer ERROR: frame failed; renderer released: {error}"),
                        );
                        renderer_state = None;
                    }
                }
                _ => {}
            }
        }
        Event::AboutToWait => {
            if renderer_state.is_some() {
                const MOVE_STEP: f32 = 0.22;
                const LOOK_STEP: f32 = 0.025;
                if pressed_keys.contains(&KeyCode::KeyW) { frame.camera.move_forward(MOVE_STEP); }
                if pressed_keys.contains(&KeyCode::KeyS) { frame.camera.move_forward(-MOVE_STEP); }
                if pressed_keys.contains(&KeyCode::KeyA) { frame.camera.move_right(-MOVE_STEP); }
                if pressed_keys.contains(&KeyCode::KeyD) { frame.camera.move_right(MOVE_STEP); }
                if pressed_keys.contains(&KeyCode::Space) { frame.camera.move_vertical(MOVE_STEP); }
                if pressed_keys.contains(&KeyCode::ShiftLeft) { frame.camera.move_vertical(-MOVE_STEP); }
                if pressed_keys.contains(&KeyCode::ArrowLeft) { frame.camera.look(-LOOK_STEP, 0.0); }
                if pressed_keys.contains(&KeyCode::ArrowRight) { frame.camera.look(LOOK_STEP, 0.0); }
                if pressed_keys.contains(&KeyCode::ArrowUp) { frame.camera.look(0.0, LOOK_STEP); }
                if pressed_keys.contains(&KeyCode::ArrowDown) { frame.camera.look(0.0, -LOOK_STEP); }

                let now = Instant::now();
                let deadline = next_frame_deadline.unwrap_or(now);
                if now >= deadline {
                    if let Some((window, _)) = renderer_state.as_ref() {
                        window.request_redraw();
                    }
                    let next_deadline = Instant::now() + FRAME_INTERVAL;
                    next_frame_deadline = Some(next_deadline);
                    active_loop.set_control_flow(ControlFlow::WaitUntil(next_deadline));
                } else {
                    active_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
                }
            } else {
                next_frame_deadline = None;
                active_loop.set_control_flow(ControlFlow::Wait);
            }
        }
        Event::LoopExiting => {
            log::info!("Winit Android event loop is exiting");
            book_renderer_opengl::record_android_diagnostic("Winit: event loop is exiting");
        }
        _ => {}
    });
    if let Err(error) = result {
        log::error!("Winit Android event loop returned an error: {error}");
        book_renderer_opengl::record_android_diagnostic(
            &format!("Winit ERROR: event loop returned: {error}"),
        );
    } else {
        log::info!("Winit Android event loop returned normally");
        book_renderer_opengl::record_android_diagnostic(
            "Winit: event loop returned normally",
        );
    }
}
