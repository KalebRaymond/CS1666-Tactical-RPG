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
}