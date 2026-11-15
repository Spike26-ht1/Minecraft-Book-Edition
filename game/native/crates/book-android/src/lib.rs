//! Android JNI boundary for the Book Edition game runtime.

#[cfg(target_os = "android")]
mod android {
    use std::ffi::{c_void, CString};
    use std::sync::{Mutex, OnceLock};

    use book64_gles::GlesRenderer;
    use jni::objects::JObject;
    use jni::sys::{jfloat, jint, jstring};
    use jni::JNIEnv;

    static RENDERER: OnceLock<Mutex<Option<GlesRenderer>>> = OnceLock::new();

    fn renderer_slot() -> &'static Mutex<Option<GlesRenderer>> {
        RENDERER.get_or_init(|| Mutex::new(None))
    }

    #[link(name = "EGL")]
    extern "C" {
        fn eglGetProcAddress(procname: *const i8) -> *const c_void;
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeSurfaceCreated(
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
    ) {
        let result = unsafe {
            GlesRenderer::initialize(|name| {
                let name = CString::new(name).expect("OpenGL symbol name");
                eglGetProcAddress(name.as_ptr())
            })
        };

        let mut slot = renderer_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        match result {
            Ok(renderer) => *slot = Some(renderer),
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
        let mut slot = renderer_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(renderer) = slot.as_mut() {
            unsafe { renderer.resize(width, height) };
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeDrawFrame(
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
    ) {
        let mut slot = renderer_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(renderer) = slot.as_mut() {
            unsafe { renderer.draw() };
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeSetMovement(
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
        forward: jfloat,
        strafe: jfloat,
        vertical: jfloat,
    ) {
        let mut slot = renderer_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(renderer) = slot.as_mut() {
            renderer.set_movement(forward, strafe, vertical);
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeLook(
        _env: JNIEnv<'_>,
        _this: JObject<'_>,
        yaw_delta: jfloat,
        pitch_delta: jfloat,
    ) {
        let mut slot = renderer_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(renderer) = slot.as_mut() {
            renderer.rotate_camera(yaw_delta, pitch_delta);
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_bookedition_game_NativeBridge_nativeGetDebugInfo(
        mut env: JNIEnv<'_>,
        _this: JObject<'_>,
    ) -> jstring {
        let slot = renderer_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let info = slot
            .as_ref()
            .map(|renderer| renderer.debug_info())
            .unwrap_or_else(|| "Book64 renderer not initialized".to_owned());

        match env.new_string(info) {
            Ok(value) => value.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    }
}

#[cfg(not(target_os = "android"))]
pub fn android_target_only() -> &'static str {
    "The JNI entry points are compiled only for Android targets."
}
