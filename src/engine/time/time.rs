use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Time {
	delta: Duration,
	last_update: Option<Instant>,
	delta_seconds_f64: f64,
	delta_seconds: f32,
	seconds_since_startup: f64,
	time_since_startup: Duration,
	startup: Instant,
}

impl Default for Time {
	fn default() -> Self {
		Self {
			delta: Duration::from_secs(0),
			last_update: None,
			delta_seconds_f64: 0.0,
			delta_seconds: 0.0,
			seconds_since_startup: 0.0,
			time_since_startup: Duration::from_secs(0),
			startup: Instant::now(),
		}
	}
}

impl Time {
	pub fn update(&mut self) {
		self.update_with_instant(Instant::now());
	}

	pub fn update_with_instant(&mut self, instant: Instant) {
		if let Some(last_update) = self.last_update {
			self.delta = instant - last_update;
			self.delta_seconds_f64 = self.delta.as_secs_f64();
			self.delta_seconds = self.delta.as_secs_f32();
		}

		self.time_since_startup = instant - self.startup;
		self.seconds_since_startup = self.time_since_startup.as_secs_f64();
		self.last_update = Some(instant);
	}

	pub fn delta(&self) -> Duration {
		self.delta
	}

	pub fn delta_seconds(&self) -> f32 {
		self.delta_seconds
	}

	pub fn delta_seconds_f64(&self) -> f64 {
		self.delta_seconds_f64
	}

	pub fn seconds_since_startup(&self) -> f64 {
		self.seconds_since_startup
	}

	pub fn time_since_startup(&self) -> Duration {
		self.time_since_startup
	}

	pub fn startup(&self) -> Instant {
		self.startup
	}

	pub fn last_update(&self) -> Option<Instant> {
		self.last_update
	}
}