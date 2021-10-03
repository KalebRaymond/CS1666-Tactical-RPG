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