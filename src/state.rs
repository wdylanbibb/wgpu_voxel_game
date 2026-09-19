use std::{collections::HashMap, sync::Arc};

use winit::{
    event::{MouseButton, MouseScrollDelta},
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::Window,
};

use crate::{
    camera, game, input, renderer,
    world::{
        World,
        generation::{self},
        meshing::mesh_chunk,
    },
};

pub struct State {
    window: Arc<Window>,
    renderer: renderer::Renderer,
    game: game::Game,
    camera: camera::CameraState,
    input: input::InputState,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let renderer = renderer::Renderer::new(window.clone()).await?;

        let game = game::Game {
            world: {
                let mut chunks = HashMap::new();

                let generator = generation::TerrainGenerator::new(
                    1337,
                    generation::TerrainConfig {
                        base_height: 4,
                        height_amplitude: 12,
                        soil_depth: 3,
                    },
                );

                for x in -4..4 {
                    for y in -2..2 {
                        for z in -4..4 {
                            let chunk_pos = cgmath::Vector3::new(x, y, z);

                            let chunk = generator.generate_chunk(chunk_pos);

                            chunks.insert(chunk_pos, chunk);
                        }
                    }
                }

                World::from_chunks(chunks)
            },
        };

        let camera = camera::CameraState::new(
            &renderer.device,
            &renderer.layouts.camera,
            renderer.config.width,
            renderer.config.height,
        );

        let input = input::InputState::default();

        Ok(Self {
            window,
            renderer,
            game,
            camera,
            input,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.renderer.resize(width, height);
            self.camera.resize(&self.renderer.queue, width, height);
            self.window.request_redraw();
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        self.game.update(dt);

        self.sync_chunk_meshes();

        self.camera.update(&self.renderer.queue, dt);
        self.renderer.update(dt);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        self.renderer.render(&self.camera)
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: KeyCode, pressed: bool) {
        if self.camera.handle_key(key, pressed) {
            return;
        }

        if key == KeyCode::Escape && pressed {
            event_loop.exit();
        }
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        if button == MouseButton::Left {
            self.input.mouse_pressed = pressed;
        }
    }

    pub fn handle_mouse_motion(&mut self, dx: f64, dy: f64) {
        if self.input.mouse_pressed {
            self.camera.handle_mouse(dx, dy);
        }
    }

    pub fn handle_mouse_scroll(&mut self, delta: &MouseScrollDelta) {
        self.camera.handle_scroll(delta);
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn sync_chunk_meshes(&mut self) {
        let dirty_positions = self.game.world.take_dirty_meshes();

        for chunk_pos in dirty_positions {
            if self.game.world.contains_chunk(chunk_pos) {
                let mesh = mesh_chunk(&self.game.world, chunk_pos);

                self.renderer.upload_chunk_mesh(chunk_pos, mesh);
            } else {
                self.renderer.remove_chunk_mesh(chunk_pos);
            }
        }
    }
}
