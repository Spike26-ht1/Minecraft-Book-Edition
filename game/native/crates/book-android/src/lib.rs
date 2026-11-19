//! Android JNI boundary and runtime composition for Book Edition.

#[cfg(target_os = "android")]
mod android {
    use std::ffi::{c_char, c_void, CString};
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use std::time::Instant;

    use book64_gles::GlesRenderer;
    use book_game::GameSession;
    use book_world::World;
    use jni::objects::{JObject, JString};
    use jni::sys::{jfloat, jint, jlong, jstring};
    use jni::JNIEnv;

    struct GameRuntime {
        session: GameSession,
        renderer: GlesRenderer,
        last_frame: Instant,
        save_path: Option<PathBuf>,
        save_enabled: bool,
        world_mesh_dirty: bool,
    }

    fn persist_world(runtime: &GameRuntime) -> Result<(), String> {
        if !runtime.save_enabled {
            return Err(
                "saving is disabled because the existing world could not be loaded".to_owned(),
            );
        }
        let path = runtime
            .save_path
            .as_ref()
            .ok_or("world save path is unavailable")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        let temporary = path.with_extension("bookworld.tmp");
        let bytes = runtime.session.world().encode();
        let mut file = fs::File::create(&temporary).map_err(|error| error.to_string())?;
        file.write_all(&bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        drop(file);
        fs::rename(&temporary, path).map_err(|error| error.to_string())
    }

    static RUNTIME: OnceLock<Mutex<Option<GameRuntime>>> = OnceLock::new();

    fn runtime_slot() -> &'static Mutex<Option<GameRuntime>> {
        RUNTIME.get_or_init(|| Mutex::new(None))
    }

    #[link(name = "EGL")]
    extern "C" {
        fn eglGetProcAddress(procname: *const c_char) -> *const c_void;
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeSurfaceCreated(
        mut env: JNIEnv<'_>,
        _this: JObject<'_>,
        seed: jlong,
        world_path: JString<'_>,
    ) {
        let seed = seed as u64;
        let path_text: String = env
            .get_string(&world_path)
            .map(|value| value.into())
            .unwrap_or_default();
        let save_path = if path_text.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(path_text))
        };

        let mut save_enabled = true;
        let session = if let Some(path) = save_path.as_ref().filter(|path| path.exists()) {
            match fs::read(path) {
                Ok(bytes) => match World::decode(&bytes) {
                    Ok(world) if world.seed() == seed => GameSession::from_world(world),
                    Ok(_) => {
                        save_enabled = false;
                        eprintln!("Book World save seed does not match the selected world");
                        GameSession::new(seed)
                    }
                    Err(error) => {
                        save_enabled = false;
                        eprintln!("Book World save could not be decoded: {error}");
                        GameSession::new(seed)
                    }
                },
                Err(error) => {
                    save_enabled = false;
                    eprintln!("Book World save could not be read: {error}");
                    GameSession::new(seed)
                }
            }
        } else {
            GameSession::new(seed)
        };
        let renderer_result = unsafe {
            GlesRenderer::initialize(
                |name| {
                    let name = CString::new(name).expect("OpenGL symbol name");
                    eglGetProcAddress(name.as_ptr())
                },
                session.world(),
            )
        };

        let mut slot = runtime_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        match renderer_result {
            Ok(renderer) => {
                *slot = Some(GameRuntime {
                    session,
                    renderer,
                    last_frame: Instant::now(),
                    save_path,
                    save_enabled,
                    world_mesh_dirty: false,
                });
            }
            Err(error) => {
                *slot = None;
                eprintln!("Book64 initialization failed: {error}");
            }
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeSurfaceChanged(
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
        width: jint,
        height: jint,
    ) {
        let mut slot = runtime_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(runtime) = slot.as_mut() {
            unsafe { runtime.renderer.resize(width, height) };
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeDrawFrame(
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
    ) {
        let mut slot = runtime_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(runtime) = slot.as_mut() {
            let now = Instant::now();
            let delta_seconds = now.duration_since(runtime.last_frame).as_secs_f32();
            runtime.last_frame = now;
            runtime.session.advance_frame(delta_seconds);
            if runtime.world_mesh_dirty {
                unsafe { runtime.renderer.update_world(runtime.session.world()) };
                runtime.world_mesh_dirty = false;
            }
            let render_state = runtime.session.render_state();
            unsafe { runtime.renderer.draw(render_state) };
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeSetMovement(
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
        forward: jfloat,
        strafe: jfloat,
    ) {
        let mut slot = runtime_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(runtime) = slot.as_mut() {
            runtime.session.set_movement(forward, strafe);
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeLook(
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
        yaw_delta: jfloat,
        pitch_delta: jfloat,
    ) {
        let mut slot = runtime_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(runtime) = slot.as_mut() {
            runtime.session.rotate_view(yaw_delta, pitch_delta);
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeJump(
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
    ) {
        let mut slot = runtime_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(runtime) = slot.as_mut() {
            runtime.session.request_jump();
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeBreakBlock(
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
    ) {
        let mut slot = runtime_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(runtime) = slot.as_mut() {
            if runtime.session.break_targeted_block() {
                runtime.world_mesh_dirty = true;
            }
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativePlaceBlock(
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
    ) {
        let mut slot = runtime_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(runtime) = slot.as_mut() {
            if runtime.session.place_targeted_block() {
                runtime.world_mesh_dirty = true;
            }
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeSaveWorld(
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
    ) {
        let slot = runtime_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(runtime) = slot.as_ref() {
            if let Err(error) = persist_world(runtime) {
                eprintln!("Book World save failed: {error}");
            }
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeGetDebugInfo(
        mut env: JNIEnv<'_>,
        _this: JObject<'_>,
    ) -> jstring {
        let slot = runtime_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let info = slot
            .as_ref()
            .map(|runtime| runtime.renderer.debug_info())
            .unwrap_or_else(|| "Book64 runtime not initialized".to_owned());

        match env.new_string(info) {
            Ok(value) => value.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    }
}

#[cfg(not(target_os = "android"))]
pub fn android_target_only() -> &'static str {
    "The JNI runtime is compiled only for Android targets."
}
