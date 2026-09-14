use crate::world;

pub struct Game {
    pub world: world::World,
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: world::World::new()
        }
    }

    pub fn update(&self, dt: std::time::Duration) {
        
    }
}
