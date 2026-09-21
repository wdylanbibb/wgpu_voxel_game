mod camera;
mod resources;
mod app;
mod state;
mod renderer;
mod input;
mod world;
mod game;
mod player;

pub use app::run;

#[cfg(target_arch = "wasm32")]
pub use app::run_web;
