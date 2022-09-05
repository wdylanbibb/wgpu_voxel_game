use std::time::Duration;

use crate::engine::time::stopwatch::Stopwatch;

#[derive(Clone, Debug, Default)]
pub struct Timer {
	stopwatch: Stopwatch,
	duration: Duration,
	repeating: bool,
	finished: bool,
	times_finished_this_tick: u32,
}

impl Timer {
	pub fn new(duration: Duration, repeating: bool) -> Self {
		Self {
			duration,
			repeating,
			..Default::default()
		}
	}

	pub fn from_seconds(seconds: f32, repeating: bool) -> Self {
		Self::new(Duration::from_secs_f32(seconds), repeating)
	}

	pub fn finished(&self) -> bool {
		self.finished
	}

	pub fn just_finished(&self) -> bool {
		self.times_finished_this_tick > 0
	}

	pub fn elapsed(&self) -> Duration {
		self.stopwatch.elapsed()
	}

	pub fn elapsed_secs(&self) -> f32 {
		self.stopwatch.elapsed_secs()
	}

	pub fn set_elapsed(&mut self, time: Duration) {
		self.stopwatch.set_elapsed(time)
	}

	pub fn duration(&self) -> Duration {
		self.duration
	}

	pub fn set_duration(&mut self, duration: Duration) {
		self.duration = duration
	}

	pub fn repeating(&self) -> bool {
		self.repeating
	}

	pub fn set_repeating(&mut self, repeating: bool) {
		if !self.repeating && repeating && self.finished {
			self.stopwatch.reset();
			self.finished = self.just_finished();
		}
		self.repeating = repeating;
	}

	pub fn tick(&mut self, delta: Duration) -> &Self {
		if self.paused() {
			self.times_finished_this_tick = 0;
			if self.repeating() {
				self.finished = false;
			}
			return self;
		}

		if !self.repeating() && self.finished() {
			self.times_finished_this_tick = 0;
			return self;
		}

		self.stopwatch.tick(delta);
		self.finished = self.elapsed() >= self.duration();

		if self.finished {
			if self.repeating() {
				self.times_finished_this_tick = (self.elapsed().as_nanos() / self.duration().as_nanos()) as u32;
				self.set_elapsed(self.elapsed() - self.duration() * self.times_finished_this_tick);
			} else {
				self.times_finished_this_tick = 1;
				self.set_elapsed(self.duration());
			}
		} else {
			self.times_finished_this_tick = 0;
		}

		self
	}

	pub fn pause(&mut self) {
		self.stopwatch.pause();
	}

	pub fn unpause(&mut self) {
		self.stopwatch.unpause();
	}

	pub fn paused(&self) -> bool {
		self.stopwatch.paused()
	}

	pub fn reset(&mut self) {
		self.stopwatch.reset();
		self.finished = false;
		self.times_finished_this_tick = 0;
	}

	pub fn percent(&self) -> f32 {
		self.elapsed().as_secs_f32() / self.duration().as_secs_f32()
	}

	pub fn percent_remaining(&self) -> f32 {
		1.0 - self.percent()
	}

	pub fn times_finished_this_tick(&self) -> u32 {
		self.times_finished_this_tick
	}
}