/// Connectable rectangle
///
/// Put a weird prefix to avoid name collision with egui's Rect
pub(crate) struct ConRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub left_con: Option<usize>,
    pub right_con: Option<usize>,
    pub top_con: Option<usize>,
    pub bottom_con: Option<usize>,
}

impl ConRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> ConRect {
        Self {
            x,
            y,
            width,
            height,
            left_con: None,
            right_con: None,
            top_con: None,
            bottom_con: None,
        }
    }

    pub fn connectors(&self) -> Vec<usize> {
        let mut ret = vec![];
        if let Some(v) = self.left_con {
            ret.push(v);
        }
        self.right_con.map(|v| ret.push(v));
        self.top_con.map(|v| ret.push(v));
        self.bottom_con.map(|v| ret.push(v));
        ret
    }
}
