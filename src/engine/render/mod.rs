use crate::engine::engine::{CoreStage, Engine, Module};
use crate::engine::render::renderer::Renderer;
use crate::engine::window::window::WindowContainer;

pub mod renderer;

struct RenderModule;

impl Module for RenderModule {
	fn build(&self, engine: &mut Engine) {
		let window = engine.world.non_send_resource::<WindowContainer>();

		engine
			.insert_non_send_resource(Renderer::new(&window.0))
			.add_system_to_stage(CoreStage::Render, render_system);
	}
}

fn render_system() {}