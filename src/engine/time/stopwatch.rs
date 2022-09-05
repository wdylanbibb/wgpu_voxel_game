use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub struct Stopwatch {
	elapsed: Duration,
	paused: bool,
}

impl Stopwatch {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn elapsed(&self) -> Duration {
		self.elapsed
	}

	pub fn elapsed_secs(&self) -> f32 {
		self.elapsed().as_secs_f32()
	}

	pub fn set_elapsed(&mut self, time: Duration) {
		self.elapsed = time;
	}

	pub fn tick(&mut self, delta: Duration) -> &Self {
		if !self.paused() {
			self.elapsed += delta;
		}
		self
	}

	pub fn pause(&mut self) {
		self.paused = true;
	}

	pub fn unpause(&mut self) {
		self.paused = false;
	}

	pub fn paused(&self) -> bool {
		self.paused
	}

	pub fn reset(&mut self) {
		self.elapsed = Duration::from_secs(0);
	}
}