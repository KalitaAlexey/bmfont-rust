/// Rectangle
#[derive(Clone, Debug)]
pub struct Rect {
    /// Minimum x
    pub x: i32,
    /// Minimum y
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn max_x(&self) -> i32 {
        self.x + self.width as i32 - 1
    }

    pub fn max_y(&self) -> i32 {
        self.y + self.height as i32 - 1
    }
}
