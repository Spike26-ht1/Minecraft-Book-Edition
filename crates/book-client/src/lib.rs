#![cfg(target_os = "android")]

use book_renderer_api::{ClearColor, RenderFrame, RendererBackend};
use book_renderer_opengl::OpenGlRenderer;
use glium::winit::event::{ElementState, Event, TouchPhase, WindowEvent};
use glium::winit::keyboard::{KeyCode, PhysicalKey};
use glium::winit::platform::android::activity::AndroidApp;
use glium::winit::platform::android::EventLoopBuilderExtAndroid;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const PACKAGE_ID: &str = "com.spike26.minecraftbookedition";

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
    log::info!("Minecraft Book Edition Pre-Alpha 1.2.0: android_main entered");
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

    let frame = RenderFrame { clear_color: ClearColor::SKY };
    let mut renderer_state = None;
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
                            &format!("Renderer ERROR: initialization failed: {error}"),
                        );
                        active_loop.exit();
                    }
                }
            }
        }
        Event::Suspended => {
            log::info!("Android lifecycle: Suspended; releasing renderer");
            book_renderer_opengl::record_android_diagnostic(
                "Android lifecycle: Suspended; releasing renderer",
            );
            renderer_state = None;
        }
        Event::WindowEvent { window_id, event } => {
            if let Some((window, renderer)) = renderer_state.as_mut() {
                if window.id() != window_id { return; }
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
                        if touch.location.x >= size.width.saturating_sub(96) as f64
                            && touch.location.y <= 64.0
                        {
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
            if let Some((window, _)) = renderer_state.as_ref() { window.request_redraw(); }
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
