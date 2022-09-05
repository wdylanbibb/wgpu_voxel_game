use std::hash::Hash;

use hashbrown::HashSet;

pub struct Input<T: Eq + Hash> {
	pressed: HashSet<T>,
	just_pressed: HashSet<T>,
	just_released: HashSet<T>,
}

impl<T: Eq + Hash> Default for Input<T> {
	fn default() -> Self {
		Self {
			pressed: Default::default(),
			just_pressed: Default::default(),
			just_released: Default::default(),
		}
	}
}

impl<T> Input<T>
	where
		T: Copy + Eq + Hash,
{
	pub fn press(&mut self, input: T) {
		if self.pressed.insert(input) {
			self.just_pressed.insert(input);
		}
	}

	pub fn pressed(&self, input: T) -> bool {
		self.pressed.contains(&input)
	}

	pub fn release(&mut self, input: T) {
		if self.pressed.remove(&input) {
			self.just_released.insert(input);
		}
	}

	pub fn release_all(&mut self) {
		self.just_released.extend(self.pressed.drain());
	}

	pub fn just_pressed(&self, input: T) -> bool {
		self.just_pressed.contains(&input)
	}

	pub fn any_just_pressed(&self, inputs: impl IntoIterator<Item=T>) -> bool {
		inputs.into_iter().any(|it| self.just_pressed(it))
	}

	pub fn clear_just_pressed(&mut self, input: T) -> bool {
		self.just_pressed.remove(&input)
	}

	pub fn just_released(&self, input: T) -> bool {
		self.just_released.contains(&input)
	}

	pub fn any_just_released(&self, inputs: impl IntoIterator<Item=T>) -> bool {
		inputs.into_iter().any(|it| self.just_released(it))
	}

	pub fn clear_just_released(&mut self, input: T) -> bool {
		self.just_released.remove(&input)
	}

	pub fn reset(&mut self, input: T) {
		self.pressed.remove(&input);
		self.just_pressed.remove(&input);
		self.just_released.remove(&input);
	}

	pub fn clear(&mut self) {
		self.just_pressed.clear();
		self.just_released.clear();
	}

	pub fn get_pressed(&self) -> impl ExactSizeIterator<Item=&T> {
		self.pressed.iter()
	}

	pub fn get_just_pressed(&self) -> impl ExactSizeIterator<Item=&T> {
		self.just_pressed.iter()
	}

	pub fn get_just_released(&self) -> impl ExactSizeIterator<Item=&T> {
		self.just_released.iter()
	}
}