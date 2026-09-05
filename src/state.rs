use std::sync::Arc;

use winit::{
    event::{MouseButton, MouseScrollDelta},
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::Window,
};

use crate::{camera, input, light, renderer, scene};

pub struct State {
    window: Arc<Window>,
    renderer: renderer::Renderer,
    scene: scene::Scene,
    camera: camera::CameraState,
    light: light::LightState,
    input: input::InputState,
}

impl State {
    // We don't need this to be async right now,
    // but we will in the next tutorial
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let renderer = renderer::Renderer::new(window.clone()).await?;

        let scene = scene::Scene::new(
            &renderer.device,
            &renderer.queue,
            &renderer.layouts.material,
        )
        .await?;

        let camera = camera::CameraState::new(
            &renderer.device,
            &renderer.layouts.camera,
            renderer.config.width,
            renderer.config.height,
        );

        let light = light::LightState::new(&renderer.device, &renderer.layouts.light);

        let input = input::InputState::default();

        Ok(Self {
            window,
            renderer,
            scene,
            camera,
            light,
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
        self.camera.update(&self.renderer.queue, dt);
        self.light.update(&self.renderer.queue, dt);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        self.renderer.render(&self.scene, &self.camera, &self.light)
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
}
