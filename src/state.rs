use std::sync::Arc;

use winit::{
    event::{MouseButton, MouseScrollDelta},
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::{CursorGrabMode, Window},
};

use crate::{
    camera, game,
    input::{self, EditAction},
    player, raycast, renderer,
    world::{block::BlockId, meshing::mesh_chunk},
};

const MAX_PHYSICS_STEPS_PER_FRAME: usize = 8;
const PLACEABLE_BLOCKS: [BlockId; 5] = [
    BlockId::Grass,
    BlockId::Dirt,
    BlockId::Stone,
    BlockId::Sand,
    BlockId::Snow,
];

pub struct State {
    window: Arc<Window>,
    renderer: renderer::Renderer,
    game: game::Game,
    camera: camera::CameraState,
    input: input::InputState,
    selected_block: usize,
    targeted_block: Option<cgmath::Vector3<i32>>,
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
            selected_block: 1,
            targeted_block: None,
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

        let selected_block = PLACEABLE_BLOCKS[self.selected_block];
        for action in self.input.take_edit_actions().collect::<Vec<_>>() {
            let ray = self.camera.view_ray();
            match action {
                EditAction::Break => self.game.break_block(ray),
                EditAction::Place => self.game.place_block(ray, selected_block),
            };
        }

        self.targeted_block = raycast::raycast(
            &self.game.world,
            self.camera.view_ray(),
            game::BLOCK_INTERACTION_REACH,
        )
        .map(|hit| hit.block_position);

        self.camera.upload(&self.renderer.queue);

        self.sync_chunk_meshes();
        self.renderer.update(dt);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        self.renderer.render(
            &self.camera,
            self.targeted_block,
            self.input.cursor_captured,
        )
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
        if !self.input.cursor_captured {
            return;
        }

        let amount = match delta {
            MouseScrollDelta::LineDelta(_, y) => *y,
            MouseScrollDelta::PixelDelta(position) => position.y as f32,
        };

        if amount > 0.0 {
            self.selected_block =
                (self.selected_block + PLACEABLE_BLOCKS.len() - 1) % PLACEABLE_BLOCKS.len();
        } else if amount < 0.0 {
            self.selected_block = (self.selected_block + 1) % PLACEABLE_BLOCKS.len();
        }
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
