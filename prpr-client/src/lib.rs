mod billboard;
mod cloud;
mod data;
mod scene;
mod task;

use anyhow::Result;
use data::Data;
use macroquad::prelude::*;
use prpr::{build_conf, time::TimeManager, Main};
use scene::MainScene;
use std::sync::{mpsc, Mutex};

static MESSAGES_TX: Mutex<Option<mpsc::Sender<bool>>> = Mutex::new(None);
static DATA_PATH: Mutex<Option<String>> = Mutex::new(None);
pub static mut DATA: Option<Data> = None;

pub fn set_data(data: Data) {
    unsafe {
        DATA = Some(data);
    }
}

pub fn get_data() -> Option<&'static Data> {
    unsafe { DATA.as_ref() }
}

pub fn get_data_mut() -> Option<&'static mut Data> {
    unsafe { DATA.as_mut() }
}

pub fn save_data() -> Result<()> {
    std::fs::write(format!("{}/data.json", dir::root()?), serde_json::to_string(get_data().as_ref().unwrap())?)?;
    Ok(())
}

mod dir {
    use anyhow::Result;

    use crate::DATA_PATH;

    fn ensure(s: &str) -> Result<String> {
        let s = format!("{}/{}", DATA_PATH.lock().unwrap().as_ref().map(|it| it.as_str()).unwrap_or("."), s);
        if !std::fs::metadata(&s).is_ok() {
            std::fs::create_dir_all(&s)?;
        }
        Ok(s)
    }

    pub fn root() -> Result<String> {
        ensure("data")
    }

    pub fn charts() -> Result<String> {
        ensure("data/charts")
    }

    pub fn custom_charts() -> Result<String> {
        ensure("data/charts/custom")
    }

    pub fn downloaded_charts() -> Result<String> {
        ensure("data/charts/download")
    }
}

async fn the_main() -> Result<()> {
    set_pc_assets_folder("assets");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();

    if cfg!(target_os = "ios") {
        *DATA_PATH.lock().unwrap() = Some("./Documents".to_owned());
    }

    let dir = dir::root()?;
    let mut data: Data = std::fs::read_to_string(format!("{dir}/data.json"))
        .map_err(anyhow::Error::new)
        .and_then(|s| Ok(serde_json::from_str(&s)?))
        .unwrap_or_default();
    data.init().await?;
    set_data(data);
    save_data()?;

    let rx = {
        let (tx, rx) = mpsc::channel();
        *MESSAGES_TX.lock().unwrap() = Some(tx);
        rx
    };

    let _ = prpr::ui::FONT.set(load_ttf_font("font.ttf").await?);

    let mut main = Main::new(Box::new(MainScene::new().await?), TimeManager::default(), None)?;
    'app: loop {
        main.update()?;
        main.render()?;
        if let Ok(paused) = rx.try_recv() {
            if paused {
                main.pause()?;
            } else {
                main.resume()?;
            }
        }
        if main.should_exit() {
            break 'app;
        }

        next_frame().await;
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn quad_main() {
    macroquad::Window::from_config(build_conf(), async {
        if let Err(err) = the_main().await {
            error!("Error: {:?}", err);
        }
    });
}

#[cfg(target_os = "android")]
unsafe fn string_from_java(env: *mut ndk_sys::JNIEnv, s: ndk_sys::jstring) -> String {
    let get_string_utf_chars = (**env).GetStringUTFChars.unwrap();
    let release_string_utf_chars = (**env).ReleaseStringUTFChars.unwrap();

    let ptr = (get_string_utf_chars)(env, s, ::std::ptr::null::<ndk_sys::jboolean>() as _);
    let res = std::ffi::CStr::from_ptr(ptr).to_str().unwrap().to_owned();
    (release_string_utf_chars)(env, s, ptr);

    res
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_quad_1native_QuadNative_prprActivityOnPause(_: *mut std::ffi::c_void, _: *const std::ffi::c_void) {
    let _ = MESSAGES_TX.lock().unwrap().as_mut().unwrap().send(true);
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_quad_1native_QuadNative_prprActivityOnResume(_: *mut std::ffi::c_void, _: *const std::ffi::c_void) {
    if let Some(tx) = MESSAGES_TX.lock().unwrap().as_mut() {
        let _ = tx.send(false);
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn Java_quad_1native_QuadNative_setDataPath(_: *mut std::ffi::c_void, _: *const std::ffi::c_void, path: ndk_sys::jstring) {
    let env = crate::miniquad::native::attach_jni_env();
    *DATA_PATH.lock().unwrap() = Some(string_from_java(env, path));
}

#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn Java_quad_1native_QuadNative_setDpi(_: *mut std::ffi::c_void, _: *const std::ffi::c_void, dpi: ndk_sys::jint) {
    prpr::core::DPI_VALUE.store(dpi as _, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn Java_quad_1native_QuadNative_setChosenFile(_: *mut std::ffi::c_void, _: *const std::ffi::c_void, file: ndk_sys::jstring) {
    use scene::CHOSEN_FILE;

    let env = crate::miniquad::native::attach_jni_env();
    *CHOSEN_FILE.lock().unwrap() = Some(string_from_java(env, file));
}
