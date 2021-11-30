use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseState;

use std::collections::HashSet;

pub struct Input {
    pub mouse_state: MouseState,
	pub left_clicked: bool,
	pub left_held: bool,
	pub right_clicked: bool,
	pub right_held: bool,

	pub mouse_x: i32,
	pub mouse_x_old: i32,
	pub mouse_y: i32,
	pub mouse_y_old: i32,

    pub keystate: HashSet<Keycode>,
}

impl Input {
    pub fn new(event_pump: &sdl2::EventPump) -> Input {
        Input {
            mouse_state: event_pump.mouse_state(),
            left_clicked: false,
            left_held: false,
            right_clicked: false,
			right_held: false,
			mouse_x: -1,
			mouse_x_old: -1,
			mouse_y: -1,
			mouse_y_old: -1,
            keystate: HashSet::new(),
        }
    }

    pub fn update(&mut self, event_pump: &sdl2::EventPump) {		
		//Record key inputs
		self.keystate = event_pump
		.keyboard_state()
		.pressed_scancodes()
		.filter_map(Keycode::from_scancode)
		.collect();

		self.mouse_state = event_pump.mouse_state();
		self.mouse_x_old = self.mouse_x;
		self.mouse_y_old = self.mouse_y;
		self.mouse_x = self.mouse_state.x();
		self.mouse_y = self.mouse_state.y();

		if self.mouse_x_old < 0 || self.mouse_y_old < 0 {
			self.mouse_x_old = self.mouse_x;
			self.mouse_y_old = self.mouse_y;
		}

        //Check if left mouse button was pressed this frame
		if self.mouse_state.left() {
			if  !self.left_held {
				self.left_clicked = true;
				self.left_held = true;
			}
			else {
				self.left_clicked = false;
			}
		}
		else {
			self.left_clicked = false;
			self.left_held = false;
		}

		//Check if right mouse button was pressed this frame
		if self.mouse_state.right() {
			if !self.right_held {
				self.right_clicked = true;
				self.right_held = true;
			}
			else {
				self.right_clicked = false;
			}
		}
		else {
			self.right_clicked = false;
			self.right_held = false;
		}
    }
}