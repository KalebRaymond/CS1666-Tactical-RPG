use std::time::Instant;

use crate::SDLCore;

pub struct DamageIndicator {
    pub value: i32,
    pub pos_x: u32,
    pub pos_y: u32,
    pub is_visible: bool,

    last_drawn: Instant,
    elapsed_time: f32,
}

impl DamageIndicator {
    pub fn new(val: i32, x: u32, y: u32) -> DamageIndicator {
        println!("DamageIndicator created at ({}, {})", x, y);

        DamageIndicator {
            value: val,
            pos_x: x, //PixelCoordinates?
            pos_y: y,
            is_visible: true,
            
            last_drawn: Instant::now(),
            elapsed_time: 0.0,
        }        
    }

    pub fn draw(&mut self, core: &mut SDLCore) {
        //Draw here

        self.elapsed_time += self.last_drawn.elapsed().as_secs_f32();
        self.last_drawn = Instant::now();

        //Set is_visible to false after 1 second. This object should now be destroyed
        if self.elapsed_time >= 1.0 {
            self.is_visible = false;
        }
    }
}