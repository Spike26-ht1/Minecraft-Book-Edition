#![cfg(target_os = "android")]

use booking_engine::{BookingEngine, PlayerController, PlayerInput};
use book_core::BlockId;
use book_renderer_api::{MobileControlsLayout, MobileControlsState, RenderFrame, RenderSettings, RendererBackend};
use book_renderer_opengl::OpenGlRenderer;
use glium::winit::event::{ElementState, Event, MouseButton, TouchPhase, WindowEvent};
use glium::winit::event_loop::ControlFlow;
use glium::winit::keyboard::{KeyCode, PhysicalKey};
use glium::winit::platform::android::activity::AndroidApp;
use android_activity::WindowManagerFlags;
use glium::winit::platform::android::EventLoopBuilderExtAndroid;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const PACKAGE_ID: &str = "com.spike26.minecraftbookedition";
fn frame_interval(fps_limit: u16) -> Option<Duration> {
    if fps_limit == 0 {
        None
    } else {
        Some(Duration::from_nanos(1_000_000_000_u64 / fps_limit as u64))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActiveTouch {
    Joystick,
    Jump,
    Sprint,
    Break,
    Place,
    Debug,
    Settings,
    Look,
}

fn joystick_axis(point: (f64, f64), layout: MobileControlsLayout) -> [f32; 2] {
    let dx = point.0 as f32 - layout.joystick_center[0];
    let dy = point.1 as f32 - layout.joystick_center[1];
    let radius = layout.joystick_radius.max(1.0);
    let mut strafe = (dx / radius).clamp(-1.0, 1.0);
    let mut forward = (-dy / radius).clamp(-1.0, 1.0);
    let magnitude = (strafe * strafe + forward * forward).sqrt();
    if magnitude > 1.0 {
        strafe /= magnitude;
        forward /= magnitude;
    }
    [strafe, forward]
}

fn perform_touch_block_action(
    engine: &mut BookingEngine,
    player: &PlayerController,
    place: bool,
) -> Option<[i32; 3]> {
    let hit = engine.raycast_blocks(player.camera_position(), player.forward(), 6.0)?;
    if place {
        if player.intersects_block(hit.adjacent) {
            return None;
        }
        engine.place_block_at(hit.adjacent, BlockId(3)).then_some(hit.adjacent)
    } else {
        engine.break_block_at(hit.position).then_some(hit.position)
    }
}

fn save_world_if_available(engine: &BookingEngine, path: Option<&Path>) {
    let Some(path) = path else { return; };
    match engine.save_world_to_path(path) {
        Ok(()) => log::info!("Booking world saved to {}", path.display()),
        Err(error) => log::error!("Could not save Booking world to {}: {error}", path.display()),
    }
}

fn save_player_if_available(player: &PlayerController, path: Option<&Path>) {
    let Some(path) = path else { return; };
    match player.save_to_path(path) {
        Ok(()) => log::info!("Booking player state saved to {}", path.display()),
        Err(error) => log::error!("Could not save player state to {}: {error}", path.display()),
    }
}

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

#[allow(unsafe_code)]
fn enable_android_immersive_fullscreen(app: &AndroidApp) {
    app.set_window_flags(
        WindowManagerFlags::FULLSCREEN | WindowManagerFlags::KEEP_SCREEN_ON,
        WindowManagerFlags::empty(),
    );

    // Hide both Android system bars for gameplay. Android can temporarily
    // reveal them when the player swipes from an edge; IMMERSIVE_STICKY then
    // lets the system hide them again automatically.
    let java_app = app.clone();
    app.run_on_java_main_thread(Box::new(move || {
        use jni::objects::JObject;
        use jni::refs::Global;
        use jni::vm::JavaVM;
        use jni::{jni_sig, jni_str, JValue};

        let vm = unsafe { JavaVM::from_raw(java_app.vm_as_ptr().cast()) };
        let result = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            // AndroidApp owns this global Activity reference. Borrow it through
            // Env instead of wrapping it as an owned JObject/Global.
            let activity_raw = java_app.activity_as_ptr() as jni::sys::jobject;
            let activity = unsafe { env.as_cast_raw::<Global<JObject>>(&activity_raw)? };
            let window = env
                .call_method(
                    activity.as_ref(),
                    jni_str!("getWindow"),
                    jni_sig!("()Landroid/view/Window;"),
                    &[],
                )?
                .l()?;
            let decor_view = env
                .call_method(
                    &window,
                    jni_str!("getDecorView"),
                    jni_sig!("()Landroid/view/View;"),
                    &[],
                )?
                .l()?;

            let sdk_int = env
                .get_static_field(
                    jni_str!("android/os/Build$VERSION"),
                    jni_str!("SDK_INT"),
                    jni_sig!("I"),
                )?
                .i()?;

            if sdk_int >= 30 {
                // Android 15+ enforces edge-to-edge for target SDK 35 apps.
                // Use the modern controller instead of deprecated View visibility flags.
                let controller = env
                    .call_method(
                        &window,
                        jni_str!("getInsetsController"),
                        jni_sig!("()Landroid/view/WindowInsetsController;"),
                        &[],
                    )?
                    .l()?;

                if !controller.is_null() {
                    let status_bars = env
                        .call_static_method(
                            jni_str!("android/view/WindowInsets$Type"),
                            jni_str!("statusBars"),
                            jni_sig!("()I"),
                            &[],
                        )?
                        .i()?;
                    let navigation_bars = env
                        .call_static_method(
                            jni_str!("android/view/WindowInsets$Type"),
                            jni_str!("navigationBars"),
                            jni_sig!("()I"),
                            &[],
                        )?
                        .i()?;

                    env.call_method(
                        &controller,
                        jni_str!("setSystemBarsBehavior"),
                        jni_sig!("(I)V"),
                        &[JValue::Int(2)], // BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
                    )?;
                    env.call_method(
                        &controller,
                        jni_str!("hide"),
                        jni_sig!("(I)V"),
                        &[JValue::Int(status_bars | navigation_bars)],
                    )?;
                    log::info!("Android immersive fullscreen applied with WindowInsetsController");
                } else {
                    log::warn!("Android WindowInsetsController is unavailable; keeping window fullscreen theme");
                }
            } else {
                // Legacy immersive mode for Android 8–10.
                const IMMERSIVE_FLAGS: i32 =
                    0x00000100 // SYSTEM_UI_FLAG_LAYOUT_STABLE
                    | 0x00000200 // SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                    | 0x00000400 // SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                    | 0x00000002 // SYSTEM_UI_FLAG_HIDE_NAVIGATION
                    | 0x00000004 // SYSTEM_UI_FLAG_FULLSCREEN
                    | 0x00001000; // SYSTEM_UI_FLAG_IMMERSIVE_STICKY

                env.call_method(
                    &decor_view,
                    jni_str!("setSystemUiVisibility"),
                    jni_sig!("(I)V"),
                    &[JValue::Int(IMMERSIVE_FLAGS)],
                )?;
                log::info!("Android legacy immersive fullscreen applied");
            }
            Ok(())
        });

        if let Err(error) = result {
            log::warn!("Could not enable Android immersive fullscreen: {error}");
        }
    }));
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
    enable_android_immersive_fullscreen(&app);
    let fullscreen_app = app.clone();
    let storage_root = match initialize_storage(&app) {
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

    let world_save_path = storage_root
        .as_ref()
        .map(|root| root.join("game/minecraftWorlds/Default/world.book"));
    let player_save_path = world_save_path
        .as_ref()
        .map(|path| path.with_file_name("player.book"));
    let mut world_save_enabled = world_save_path.is_some();
    let mut engine = BookingEngine::new();
    let mut world_loaded = false;

    if let Some(path) = world_save_path.as_ref() {
        if path.exists() {
            match engine.load_world_from_path(path) {
                Ok(()) => {
                    world_loaded = true;
                    log::info!("Loaded saved Booking world from {}", path.display());
                }
                Err(error) => {
                    log::error!("Could not load saved Booking world: {error}");
                    let backup_path = path.with_extension("book.corrupt");
                    match fs::rename(path, &backup_path) {
                        Ok(()) => log::warn!(
                            "Moved invalid world save to {}",
                            backup_path.display()
                        ),
                        Err(backup_error) => {
                            world_save_enabled = false;
                            log::error!(
                                "Could not preserve invalid world save {}; autosaving disabled: {backup_error}",
                                path.display()
                            );
                        }
                    }
                }
            }
        }
    }

    if !world_loaded {
        // Build only the spawn chunk before the window exists. The surrounding
        // 80 columns are generated incrementally after the loading screen appears.
        engine.generate_prototype_terrain(0);
        if world_save_enabled {
            save_world_if_available(&engine, world_save_path.as_deref());
        }
    }

    // Load an extracted Bedrock resource pack while the native Android splash is visible.
    let resource_pack_atlas = storage_root.as_ref().and_then(|root| {
        let pack_root = root.join("game/resource_packs");
        match book_assets::load_bedrock_terrain_atlas(&pack_root) {
            Ok(atlas) => {
                log::info!("Loaded Bedrock terrain textures from {}", pack_root.display());
                book_renderer_opengl::record_android_diagnostic("Resource packs: loaded grass, dirt and stone PNG textures");
                Some(atlas)
            }
            Err(error) => {
                log::warn!("No usable extracted Bedrock resource pack at {}: {error}", pack_root.display());
                book_renderer_opengl::record_android_diagnostic(&format!("Resource packs: using prototype atlas ({error})"));
                None
            }
        }
    });

    let mut player = match player_save_path.as_ref() {
        Some(path) if path.exists() => match PlayerController::load_from_path(&engine, path) {
            Ok(player) => {
                log::info!("Restored player position from {}", path.display());
                player
            }
            Err(error) => {
                log::warn!("Could not restore player save: {error}");
                let backup_path = path.with_extension("book.corrupt");
                if let Err(backup_error) = fs::rename(path, &backup_path) {
                    log::error!(
                        "Could not preserve invalid player save {}; using spawn: {backup_error}",
                        path.display()
                    );
                }
                PlayerController::spawn(&engine, 0, 0).unwrap_or_default()
            }
        },
        _ => PlayerController::spawn(&engine, 0, 0).unwrap_or_default(),
    };
    let mut frame = RenderFrame::default();
    frame.camera.position = player.camera_position();
    frame.camera.yaw = player.yaw();
    frame.camera.pitch = player.pitch();
    let mut renderer_state = None;
    // Retry transient EGL/window-surface failures instead of leaving a black screen.
    let mut renderer_retry_at: Option<Instant> = None;
    let mut renderer_retry_attempts: u8 = 0;
    let mut pressed_keys = HashSet::new();
    let mut active_touches: HashMap<u64, (f64, f64)> = HashMap::new();
    let mut touch_start_positions: HashMap<u64, (f64, f64)> = HashMap::new();
    let mut touch_start_times: HashMap<u64, Instant> = HashMap::new();
    let mut touch_roles: HashMap<u64, ActiveTouch> = HashMap::new();
    let mut jump_queued = false;
    let mut next_frame_deadline: Option<Instant> = None;
    let mut last_simulation_tick = Instant::now();
    let spawn_position = player.position();
    let mut last_stream_chunk = (
        (spawn_position[0].floor() as i32).div_euclid(16),
        (spawn_position[2].floor() as i32).div_euclid(16),
    );
    let mut stream_pending = true;
    let mut renderer_loading = true;
    let result = event_loop.run(move |event, active_loop| match event {
        Event::Resumed => {
            enable_android_immersive_fullscreen(&fullscreen_app);
            log::info!("Android lifecycle: Resumed");
            book_renderer_opengl::record_android_diagnostic("Android lifecycle: Resumed");
            if renderer_state.is_none() {
                log::info!("Starting OpenGL ES 3 renderer initialization");
                book_renderer_opengl::record_android_diagnostic(
                    "Renderer: starting OpenGL ES 3 initialization",
                );
                match OpenGlRenderer::new_with_atlas(
                    active_loop,
                    "Minecraft Book Edition — Pre-Alpha 1.2.0",
                    1280,
                    720,
                    &engine,
                    resource_pack_atlas.clone(),
                ) {
                    Ok((window, mut renderer)) => {
                        log::info!("OpenGL ES renderer initialized successfully");
                        book_renderer_opengl::record_android_diagnostic(
                            "Renderer: OpenGL ES renderer initialized successfully",
                        );
                        renderer.set_loading_progress(0, engine.loaded_chunk_count().max(81));
                        renderer_loading = true;
                        renderer_state = Some((window, renderer));
                        renderer_retry_at = None;
                        renderer_retry_attempts = 0;
                    }
                    Err(error) => {
                        log::error!("OpenGL ES renderer initialization failed: {error}");
                        book_renderer_opengl::record_android_diagnostic(
                            &format!("Renderer ERROR: initialization failed; scheduling automatic retry: {error}"),
                        );
                        renderer_retry_attempts = renderer_retry_attempts.saturating_add(1);
                        renderer_retry_at = Some(Instant::now() + Duration::from_secs(1));
                    }
                }
            }
            if let Some((window, _)) = renderer_state.as_ref() {
                window.request_redraw();
                if let Some(interval) = frame_interval(frame.settings.fps_limit) {
                    let deadline = Instant::now() + interval;
                    next_frame_deadline = Some(deadline);
                    active_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
                } else {
                    next_frame_deadline = None;
                    active_loop.set_control_flow(ControlFlow::Poll);
                }
            } else {
                next_frame_deadline = None;
                if let Some(retry_at) = renderer_retry_at {
                    active_loop.set_control_flow(ControlFlow::WaitUntil(retry_at));
                } else {
                    active_loop.set_control_flow(ControlFlow::Wait);
                }
            }
        }
        Event::Suspended => {
            pressed_keys.clear();
            active_touches.clear();
            touch_start_positions.clear();
            touch_start_times.clear();
            touch_roles.clear();
            jump_queued = false;
            frame.mobile_controls = MobileControlsState::default();
            if world_save_enabled {
                save_world_if_available(&engine, world_save_path.as_deref());
                save_player_if_available(&player, player_save_path.as_deref());
            }
            log::info!("Android lifecycle: Suspended; releasing renderer");
            book_renderer_opengl::record_android_diagnostic(
                "Android lifecycle: Suspended; releasing renderer",
            );
            renderer_state = None;
            renderer_retry_at = None;
            renderer_retry_attempts = 0;
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
                    if focused {
                        enable_android_immersive_fullscreen(&fullscreen_app);
                    }
                    if !focused {
                        pressed_keys.clear();
                        active_touches.clear();
                        touch_start_positions.clear();
                        touch_start_times.clear();
                        touch_roles.clear();
                        jump_queued = false;
                        frame.mobile_controls = MobileControlsState::default();
                    }
                    log::info!("Android window focus changed: focused={focused}");
                    book_renderer_opengl::record_android_diagnostic(&format!(
                        "Android window focus changed: focused={focused}"
                    ));
                }
                WindowEvent::Resized(size) => {
                    // Android may resize the native window after immersive fullscreen
                    // is applied or system bars temporarily change visibility.
                    // Keep the EGL surface and pixel-space HUD buffers in sync.
                    if size.width > 0 && size.height > 0 {
                        let resize_error = renderer_state
                            .as_mut()
                            .and_then(|(_, renderer)| renderer.resize(size.width, size.height).err());
                        if let Some(error) = resize_error {
                            log::error!("Android renderer resize failed: {error}");
                            book_renderer_opengl::record_android_diagnostic(
                                &format!("Renderer ERROR: resize failed; scheduling recovery: {error}"),
                            );
                            renderer_state = None;
                            renderer_retry_attempts = renderer_retry_attempts.saturating_add(1);
                            renderer_retry_at = Some(Instant::now() + Duration::from_secs(1));
                        }
                        if let Some((window, _)) = renderer_state.as_ref() {
                            window.request_redraw();
                        }
                    }
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
                WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                    if let Some(hit) = engine.raycast_blocks(
                        player.camera_position(), player.forward(), 6.0,
                    ) {
                        if engine.break_block_at(hit.position) {
                            if let Some((window, renderer)) = renderer_state.as_mut() {
                                let chunk_x = hit.position[0].div_euclid(16);
                                let chunk_z = hit.position[2].div_euclid(16);
                                if let Err(error) = renderer.sync_chunk(&engine, chunk_x, chunk_z) {
                                    log::error!("World mesh sync after mouse block break failed: {error}");
                                }
                                window.request_redraw();
                            }
                            if world_save_enabled {
                                save_world_if_available(&engine, world_save_path.as_deref());
                                save_player_if_available(&player, player_save_path.as_deref());
                            }
                        }
                    }
                }
                WindowEvent::Touch(touch) => {
                    let touch_id = touch.id;
                    let location = (touch.location.x, touch.location.y);
                    match touch.phase {
                        TouchPhase::Started => {
                            active_touches.insert(touch_id, location);
                            touch_start_positions.insert(touch_id, location);
                            touch_start_times.insert(touch_id, Instant::now());

                            let size = renderer_state
                                .as_ref()
                                .map(|(window, _)| window.inner_size());
                            let layout = size
                                .map(|size| MobileControlsLayout::new(size.width, size.height));

                            let role = if let Some(layout) = layout {
                                let point = [location.0 as f32, location.1 as f32];
                                if layout.settings.contains(point) {
                                    let opening_menu = !frame.settings.menu_open;
                                    frame.settings.menu_open = opening_menu;
                                    frame.mobile_controls = MobileControlsState::default();
                                    if opening_menu {
                                        // Release held movement/action touches before displaying
                                        // the settings overlay.
                                        active_touches.clear();
                                        touch_start_positions.clear();
                                        touch_start_times.clear();
                                        touch_roles.clear();
                                        jump_queued = false;
                                    }
                                    if let Some((window, _)) = renderer_state.as_ref() {
                                        window.request_redraw();
                                    }
                                    ActiveTouch::Settings
                                } else if frame.settings.menu_open {
                                    if layout.settings_close.contains(point) {
                                        frame.settings.menu_open = false;
                                    } else if let Some(index) = layout.fps_options.iter()
                                        .position(|rect| rect.contains(point))
                                    {
                                        frame.settings.fps_limit = RenderSettings::FPS_OPTIONS[index];
                                    } else if let Some(index) = layout.distance_options.iter()
                                        .position(|rect| rect.contains(point))
                                    {
                                        frame.settings.render_distance_chunks =
                                            RenderSettings::DISTANCE_OPTIONS[index];
                                    }
                                    if let Some((window, _)) = renderer_state.as_ref() {
                                        window.request_redraw();
                                    }
                                    ActiveTouch::Settings
                                } else if layout.debug.contains(point) {
                                    if let Some((window, renderer)) = renderer_state.as_mut() {
                                        renderer.toggle_debug();
                                        window.request_redraw();
                                    }
                                    ActiveTouch::Debug
                                } else if layout.joystick_contains(point) {
                                    let axis = joystick_axis(location, layout);
                                    frame.mobile_controls.strafe = axis[0];
                                    frame.mobile_controls.forward = axis[1];
                                    ActiveTouch::Joystick
                                } else if layout.jump.contains(point) {
                                    frame.mobile_controls.jump_pressed = true;
                                    jump_queued = true;
                                    ActiveTouch::Jump
                                } else if layout.sprint.contains(point) {
                                    frame.mobile_controls.sprint_pressed = true;
                                    ActiveTouch::Sprint
                                } else if layout.break_block.contains(point) {
                                    frame.mobile_controls.break_pressed = true;
                                    if let Some(changed_position) =
                                        perform_touch_block_action(&mut engine, &player, false)
                                    {
                                        if let Some((window, renderer)) = renderer_state.as_mut() {
                                            let chunk_x = changed_position[0].div_euclid(16);
                                            let chunk_z = changed_position[2].div_euclid(16);
                                            if let Err(error) = renderer.sync_chunk(&engine, chunk_x, chunk_z) {
                                                log::error!("Renderer sync after mobile mining failed: {error}");
                                            }
                                            window.request_redraw();
                                        }
                                        if world_save_enabled {
                                            save_world_if_available(&engine, world_save_path.as_deref());
                                        }
                                    }
                                    ActiveTouch::Break
                                } else if layout.place_block.contains(point) {
                                    frame.mobile_controls.place_pressed = true;
                                    if let Some(changed_position) =
                                        perform_touch_block_action(&mut engine, &player, true)
                                    {
                                        if let Some((window, renderer)) = renderer_state.as_mut() {
                                            let chunk_x = changed_position[0].div_euclid(16);
                                            let chunk_z = changed_position[2].div_euclid(16);
                                            if let Err(error) = renderer.sync_chunk(&engine, chunk_x, chunk_z) {
                                                log::error!("Renderer sync after mobile placement failed: {error}");
                                            }
                                            window.request_redraw();
                                        }
                                        if world_save_enabled {
                                            save_world_if_available(&engine, world_save_path.as_deref());
                                        }
                                    }
                                    ActiveTouch::Place
                                } else {
                                    ActiveTouch::Look
                                }
                            } else {
                                ActiveTouch::Look
                            };

                            touch_roles.insert(touch_id, role);
                            if let Some((window, _)) = renderer_state.as_ref() {
                                window.request_redraw();
                            }
                        }
                        TouchPhase::Moved => {
                            let role = touch_roles.get(&touch_id).copied().unwrap_or(ActiveTouch::Look);
                            let previous_location = active_touches.insert(touch_id, location);
                            match role {
                                ActiveTouch::Joystick => {
                                    if let Some((window, _)) = renderer_state.as_ref() {
                                        let size = window.inner_size();
                                        let layout = MobileControlsLayout::new(size.width, size.height);
                                        let axis = joystick_axis(location, layout);
                                        frame.mobile_controls.strafe = axis[0];
                                        frame.mobile_controls.forward = axis[1];
                                        window.request_redraw();
                                    }
                                }
                                ActiveTouch::Look => {
                                    if let Some(previous) = previous_location {
                                        player.look(
                                            ((location.0 - previous.0) as f32) * 0.004,
                                            -((location.1 - previous.1) as f32) * 0.004,
                                        );
                                        frame.camera.position = player.camera_position();
                                        frame.camera.yaw = player.yaw();
                                        frame.camera.pitch = player.pitch();
                                        if let Some((window, _)) = renderer_state.as_ref() {
                                            window.request_redraw();
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        TouchPhase::Ended | TouchPhase::Cancelled => {
                            let start = touch_start_positions.remove(&touch_id);
                            let started = touch_start_times.remove(&touch_id);
                            active_touches.remove(&touch_id);
                            let role = touch_roles.remove(&touch_id).unwrap_or(ActiveTouch::Look);
                            let held_for = started
                                .map(|started| started.elapsed())
                                .unwrap_or_default();
                            match role {
                                ActiveTouch::Joystick => {
                                    frame.mobile_controls.strafe = 0.0;
                                    frame.mobile_controls.forward = 0.0;
                                }
                                ActiveTouch::Jump => frame.mobile_controls.jump_pressed = false,
                                ActiveTouch::Sprint => frame.mobile_controls.sprint_pressed = false,
                                ActiveTouch::Break => frame.mobile_controls.break_pressed = false,
                                ActiveTouch::Place => frame.mobile_controls.place_pressed = false,
                                ActiveTouch::Debug | ActiveTouch::Settings | ActiveTouch::Look => {}
                            }
                            if let Some((window, _)) = renderer_state.as_ref() {
                                window.request_redraw();
                            }

                            // Keep the original world-surface gesture as a fallback:
                            // short taps mine, while a long press places a block.
                            if touch.phase == TouchPhase::Ended && role == ActiveTouch::Look {
                                if let Some(start) = start {
                                    let dx = location.0 - start.0;
                                    let dy = location.1 - start.1;
                                    if dx * dx + dy * dy < 144.0 {
                                        let changed_position = perform_touch_block_action(
                                            &mut engine,
                                            &player,
                                            held_for >= Duration::from_millis(350),
                                        );
                                        if let Some(changed_position) = changed_position {
                                            if let Some((window, renderer)) = renderer_state.as_mut() {
                                                let chunk_x = changed_position[0].div_euclid(16);
                                                let chunk_z = changed_position[2].div_euclid(16);
                                                if let Err(error) = renderer.sync_chunk(&engine, chunk_x, chunk_z) {
                                                    log::error!("Renderer sync after world tap failed: {error}");
                                                }
                                                window.request_redraw();
                                            }
                                            if world_save_enabled {
                                                save_world_if_available(&engine, world_save_path.as_deref());
                                            }
                                        }
                                    }
                                }
                            }
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
                        renderer_retry_attempts = renderer_retry_attempts.saturating_add(1);
                        renderer_retry_at = Some(Instant::now() + Duration::from_secs(1));
                    }
                }
                _ => {}
            }
        }
        Event::AboutToWait => {
            if renderer_state.is_some() {
                const LOOK_STEP: f32 = 0.025;
                let now = Instant::now();
                let delta_seconds = now.duration_since(last_simulation_tick).as_secs_f32();
                last_simulation_tick = now;

                let keyboard_forward = (pressed_keys.contains(&KeyCode::KeyW) as i8
                    - pressed_keys.contains(&KeyCode::KeyS) as i8) as f32;
                let keyboard_strafe = (pressed_keys.contains(&KeyCode::KeyD) as i8
                    - pressed_keys.contains(&KeyCode::KeyA) as i8) as f32;
                let input = if renderer_loading {
                    // Keep the spawn pose stable while the first 9x9 area is built.
                    PlayerInput::default()
                } else {
                    let forward = (keyboard_forward + frame.mobile_controls.forward).clamp(-1.0, 1.0);
                    let strafe = (keyboard_strafe + frame.mobile_controls.strafe).clamp(-1.0, 1.0);
                    PlayerInput {
                        forward,
                        strafe,
                        jump: pressed_keys.contains(&KeyCode::Space) || jump_queued,
                        sprint: pressed_keys.contains(&KeyCode::ShiftLeft)
                            || frame.mobile_controls.sprint_pressed,
                    }
                };
                jump_queued = false;
                player.update(&engine, input, delta_seconds);

                let player_position = player.position();
                let current_stream_chunk = (
                    (player_position[0].floor() as i32).div_euclid(16),
                    (player_position[2].floor() as i32).div_euclid(16),
                );
                if current_stream_chunk != last_stream_chunk {
                    last_stream_chunk = current_stream_chunk;
                    stream_pending = true;
                }

                // Generate and upload at most one missing chunk column per frame.
                // This spreads terrain creation and GPU buffer uploads across
                // multiple frames instead of blocking the render loop on a chunk ring.
                let mut world_chunk_generated = false;
                if stream_pending {
                    if let Some(position) = engine.ensure_next_prototype_chunk_around(
                        player_position[0].floor() as i32,
                        player_position[2].floor() as i32,
                        4,
                    ) {
                        world_chunk_generated = true;
                        log::debug!("Booking streamed chunk {}, {}", position.x, position.z);
                    } else {
                        stream_pending = false;
                    }
                }

                if let Some((window, renderer)) = renderer_state.as_mut() {
                    match renderer.sync_next_chunk(&engine) {
                        Ok(uploaded) => {
                            if uploaded || world_chunk_generated {
                                window.request_redraw();
                            }
                        }
                        Err(error) => log::error!("Incremental GPU chunk upload failed: {error}"),
                    }

                    if renderer_loading {
                        let loaded = renderer.loaded_gpu_chunk_count();
                        let total = engine.loaded_chunk_count().max(81);
                        renderer.set_loading_progress(loaded, total);
                        if !stream_pending && loaded >= engine.loaded_chunk_count() {
                            renderer.finish_loading();
                            renderer_loading = false;
                            log::info!("Booking world and initial chunk meshes are ready");
                            book_renderer_opengl::record_android_diagnostic(
                                "World loading: initial terrain and GPU chunk meshes are ready",
                            );
                        }
                    }
                }

                if pressed_keys.contains(&KeyCode::ArrowLeft) { player.look(-LOOK_STEP, 0.0); }
                if pressed_keys.contains(&KeyCode::ArrowRight) { player.look(LOOK_STEP, 0.0); }
                if pressed_keys.contains(&KeyCode::ArrowUp) { player.look(0.0, LOOK_STEP); }
                if pressed_keys.contains(&KeyCode::ArrowDown) { player.look(0.0, -LOOK_STEP); }

                frame.camera.position = player.camera_position();
                frame.camera.yaw = player.yaw();
                frame.camera.pitch = player.pitch();
                if let Some(interval) = frame_interval(frame.settings.fps_limit) {
                    let deadline = next_frame_deadline.unwrap_or(now);
                    if now >= deadline {
                        if let Some((window, _)) = renderer_state.as_ref() {
                            window.request_redraw();
                        }
                        let next_deadline = Instant::now() + interval;
                        next_frame_deadline = Some(next_deadline);
                        active_loop.set_control_flow(ControlFlow::WaitUntil(next_deadline));
                    } else {
                        active_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
                    }
                } else {
                    // Uncapped test mode: let Winit continuously poll and redraw.
                    next_frame_deadline = None;
                    active_loop.set_control_flow(ControlFlow::Poll);
                    if let Some((window, _)) = renderer_state.as_ref() {
                        window.request_redraw();
                    }
                }
            } else {
                next_frame_deadline = None;
                if let Some(retry_at) = renderer_retry_at {
                    if Instant::now() >= retry_at {
                        book_renderer_opengl::record_android_diagnostic("Renderer: retrying Android EGL/OpenGL initialization");
                        match OpenGlRenderer::new_with_atlas(
                            active_loop,
                            "Minecraft Book Edition — Pre-Alpha 1.2.0",
                            1280,
                            720,
                            &engine,
                            resource_pack_atlas.clone(),
                        ) {
                            Ok((window, mut renderer)) => {
                                log::info!("OpenGL ES renderer recovered after retry");
                                book_renderer_opengl::record_android_diagnostic("Renderer: automatic retry succeeded");
                                renderer.set_loading_progress(0, engine.loaded_chunk_count().max(81));
                                renderer_loading = true;
                                renderer_state = Some((window, renderer));
                                renderer_retry_at = None;
                                renderer_retry_attempts = 0;
                                if let Some((window, _)) = renderer_state.as_ref() {
                                    window.request_redraw();
                                }
                            }
                            Err(error) => {
                                renderer_retry_attempts = renderer_retry_attempts.saturating_add(1);
                                let backoff_seconds = (1_u64 << renderer_retry_attempts.min(4)).min(16);
                                renderer_retry_at = Some(Instant::now() + Duration::from_secs(backoff_seconds));
                                log::error!("OpenGL ES renderer retry {} failed: {error}", renderer_retry_attempts);
                                book_renderer_opengl::record_android_diagnostic(&format!("Renderer ERROR: retry {} failed; next retry in {backoff_seconds}s: {error}", renderer_retry_attempts));
                            }
                        }
                    }
                    if renderer_state.is_none() {
                        if let Some(retry_at) = renderer_retry_at {
                            active_loop.set_control_flow(ControlFlow::WaitUntil(retry_at));
                        }
                    }
                } else {
                    active_loop.set_control_flow(ControlFlow::Wait);
                }
            }
        }
        Event::LoopExiting => {
            if world_save_enabled {
                save_world_if_available(&engine, world_save_path.as_deref());
                save_player_if_available(&player, player_save_path.as_deref());
            }
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
