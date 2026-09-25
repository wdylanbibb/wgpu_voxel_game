use winit::keyboard::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditAction {
    Break,
    Place,
}

#[derive(Default)]
pub struct InputState {
    pub cursor_captured: bool,
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    jump_held: bool,
    jump_queued: bool,
    edit_actions: std::collections::VecDeque<EditAction>,
}

impl InputState {
    pub fn handle_key(&mut self, key: KeyCode, pressed: bool) -> bool {
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.forward = pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.backward = pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.left = pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.right = pressed;
                true
            }
            KeyCode::Space => {
                if pressed && !self.jump_held {
                    self.jump_queued = true;
                }
                self.jump_held = pressed;
                true
            }
            _ => false,
        }
    }

    pub fn movement(&self) -> cgmath::Vector2<f32> {
        cgmath::Vector2::new(
            (self.right as i32 - self.left as i32) as f32,
            (self.forward as i32 - self.backward as i32) as f32,
        )
    }

    pub fn take_jump(&mut self) -> bool {
        std::mem::take(&mut self.jump_queued)
    }

    pub fn queue_break(&mut self) {
        self.edit_actions.push_back(EditAction::Break);
    }

    pub fn queue_place(&mut self) {
        self.edit_actions.push_back(EditAction::Place);
    }

    pub fn take_edit_actions(&mut self) -> impl Iterator<Item = EditAction> + '_ {
        self.edit_actions.drain(..)
    }
}

#[cfg(test)]
mod tests {
    use super::{EditAction, InputState};

    #[test]
    fn edit_actions_keep_press_order() {
        let mut input = InputState::default();
        input.queue_break();
        input.queue_place();

        assert_eq!(
            input.take_edit_actions().collect::<Vec<_>>(),
            vec![EditAction::Break, EditAction::Place]
        );
    }
}
