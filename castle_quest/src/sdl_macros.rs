/// Centered Rect macro: constructs a rect in the middle of the screen with the designated width/height
macro_rules! centered_rect {
	($core: ident, $width: expr, $height: expr) => {
		Rect::new(
			($core.cam.width()/2 -$width/2) as i32,
			($core.cam.height()/2 - $height/2) as i32,
			$width,
			$height,
		);
	};
	// vertically center a rect; centered_rect!(core, right, _, w, h)
	($core: ident, $right: expr, _, $width: expr, $height: expr) => {
		Rect::new(
			$right,
			($core.cam.height()/2 - $height/2) as i32,
			$width,
			$height,
		);
	};
	// horizontally center a rect; centered_rect!(core, _, right, w, h)
	($core: ident, _, $top: expr, $width: expr, $height: expr) => {
		Rect::new(
			($core.cam.width()/2 -$width/2) as i32,
			$top,
			$width,
			$height,
		);
	};
}

/// Sleep thread macro: sleeps for the provided duration, but wakes up at intervals to check for escape key codes
macro_rules! sleep_poll {
	($core: ident, $waitms: expr) => {
		{
			let mut wait: u64 = $waitms as u64;
			'sleep: loop {
				if (wait < 100) {
					std::thread::sleep(std::time::Duration::from_millis(wait));
					break 'sleep;
				}

				std::thread::sleep(std::time::Duration::from_millis(100));
				wait -= 100;
				for event in $core.event_pump.poll_iter() {
					match event {
						sdl2::event::Event::Quit{..} | sdl2::event::Event::KeyDown{keycode: Some(sdl2::keyboard::Keycode::Escape), ..} => return Err("Escape keycode caught during sleep_poll".to_string()),
						_ => {},
					}
				}
			}
		}
	};
	($core: ident, $waitms: expr, $closure: expr) => {
		{
			let mut wait: u64 = $waitms as u64;
			'sleep: loop {
				if (wait < 100) {
					std::thread::sleep(std::time::Duration::from_millis(wait));
					break 'sleep;
				}

				std::thread::sleep(std::time::Duration::from_millis(100));
				wait -= 100;
				for event in $core.event_pump.poll_iter() {
					$closure(event);
				}
			}
		}
	}
}
