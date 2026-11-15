//! Android JNI boundary for the Book Edition game runtime.

#[cfg(target_os = "android")]
mod android {
    use std::ffi::{c_void, CString};
    use std::sync::{Mutex, OnceLock};

    use book64_gles::GlesRenderer;
    use jni::objects::JObject;
    use jni::sys::jint;
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
        let slot = renderer_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(renderer) = slot.as_ref() {
            unsafe { renderer.draw() };
        }
    }
}

#[cfg(not(target_os = "android"))]
pub fn android_target_only() -> &'static str {
    "The JNI entry points are compiled only for Android targets."
}
