use std::sync::Arc;

use winit::{
    event::{MouseButton, MouseScrollDelta},
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::{CursorGrabMode, Window},
};

use crate::{camera, game, input, player, renderer, world::meshing::mesh_chunk};

const MAX_PHYSICS_STEPS_PER_FRAME: usize = 8;

pub struct State {
    window: Arc<Window>,
    renderer: renderer::Renderer,
    game: game::Game,
    camera: camera::CameraState,
    input: input::InputState,
    physics_accumulator: f32,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let renderer = renderer::Renderer::new(window.clone()).await?;

        let game = game::Game::new(cgmath::Vector3::new(
            (-4..4).into(),
            (-2..2).into(),
            (-4..4).into(),
        ));

        let mut camera = camera::CameraState::new(
            &renderer.device,
            &renderer.layouts.camera,
            renderer.config.width,
            renderer.config.height,
        );

        camera.set_position(game.player.eye_position());

        let input = input::InputState::default();

        Ok(Self {
            window,
            renderer,
            game,
            camera,
            input,
            physics_accumulator: 0.0,
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
        self.camera.update_look(dt);

        let movement = self.input.movement();
        let (forward, right) = self.camera.horizonal_basis();

        self.physics_accumulator += dt.as_secs_f32().min(0.1);

        let mut steps = 0;
        while self.physics_accumulator >= player::FIXED_TIMESTEP
            && steps < MAX_PHYSICS_STEPS_PER_FRAME
        {
            let jump = self.input.take_jump();
            self.game
                .update(player::FIXED_TIMESTEP, movement, forward, right, jump);
            self.physics_accumulator -= player::FIXED_TIMESTEP;
            steps += 1;
        }

        if steps == MAX_PHYSICS_STEPS_PER_FRAME
            && self.physics_accumulator >= player::FIXED_TIMESTEP
        {
            self.physics_accumulator = 0.0;
        }

        self.camera.set_position(self.game.player.eye_position());
        self.camera.upload(&self.renderer.queue);

        self.sync_chunk_meshes();
        self.renderer.update(dt);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        self.renderer.render(&self.camera)
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: KeyCode, pressed: bool) {
        if self.input.handle_key(key, pressed) {
            return;
        }

        if key == KeyCode::Escape && pressed {
            if self.input.cursor_captured {
                self.release_cursor();
            } else {
                event_loop.exit();
            }
        }
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        if !pressed {
            return;
        }

        if !self.input.cursor_captured {
            self.capture_cursor();
            return;
        }

        match button {
            MouseButton::Left => {
                self.input.queue_break();
            }
            MouseButton::Right => {
                self.input.queue_place();
            }
            _ => {}
        }
    }

    pub fn handle_mouse_motion(&mut self, dx: f64, dy: f64) {
        if self.input.cursor_captured {
            self.camera.handle_mouse(dx, dy);
        }
    }

    pub fn handle_mouse_scroll(&mut self, delta: &MouseScrollDelta) {
        self.camera.handle_scroll(delta);
    }

    pub fn capture_cursor(&mut self) {
        let result = self
            .window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined));

        match result {
            Ok(()) => {
                self.window.set_cursor_visible(false);
                self.input.cursor_captured = true;
            }
            Err(error) => {
                log::warn!("Could not capture cursor: {error}");
            }
        }
    }

    pub fn release_cursor(&mut self) {
        if let Err(error) = self.window.set_cursor_grab(CursorGrabMode::None) {
            log::warn!("Could not release cursor: {error}");
        }

        self.window.set_cursor_visible(true);
        self.input.cursor_captured = false;
    }

    pub fn cursor_captured(&self) -> bool {
        self.input.cursor_captured
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
