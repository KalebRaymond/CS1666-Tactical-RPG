pub struct PixelCoordinates {
    pub x: u32,
    pub y: u32,
}

impl PixelCoordinates {
    pub fn from_matrix_indices(i: u32, j: u32) -> PixelCoordinates {
        PixelCoordinates { 
            x: j * crate::TILE_SIZE,
            y: i * crate::TILE_SIZE,
        }
    }

    pub fn matrix_indices_from_pixel(x: u32, y: u32, cam_x: u32, cam_y: u32) -> (u32, u32) {
        //Have to add the camera offsets because the camera is weird and cam_x and cam_y 
        //are both always less than or equal to zero
        let i = (y + cam_y) / crate::TILE_SIZE;
        let j = (x + cam_x) / crate::TILE_SIZE;
        
        (i, j)
    }
}