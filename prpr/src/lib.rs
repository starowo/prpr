mod ext;
pub use ext::poll_future;

pub mod audio;
pub mod config;
pub mod core;
pub mod fs;
pub mod info;
pub mod judge;
pub mod parse;
pub mod particle;
pub mod scene;
pub mod time;

pub use scene::Main;

pub fn build_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "prpr".to_string(),
        window_width: 1080,
        window_height: 608,
        ..Default::default()
    }
}
