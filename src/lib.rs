mod app;
mod camera;
mod game;
mod input;
mod player;
mod raycast;
mod renderer;
mod resources;
mod state;
mod ui;
mod world;

pub use app::run;

#[cfg(target_arch = "wasm32")]
pub use app::run_web;
