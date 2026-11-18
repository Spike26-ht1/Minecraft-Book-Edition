//! Android JNI boundary and runtime composition for Book Edition.

#[cfg(target_os = "android")]
mod android {
    use std::ffi::{c_char, c_void, CString};
    use std::sync::{Mutex, OnceLock};
    use std::time::Instant;

    use book64_gles::GlesRenderer;
    use book_game::GameSession;
    use jni::objects::JObject;
    use jni::sys::{jfloat, jint, jlong, jstring};
    use jni::JNIEnv;

    struct GameRuntime {
        session: GameSession,
        renderer: GlesRenderer,
        last_frame: Instant,
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
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
        seed: jlong,
    ) {
        let session = GameSession::new(seed as u64);
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
