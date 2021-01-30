/// The dimension of a canvas
pub struct Dimension {
    pub width: u32,
    pub height: u32,
}

impl Dimension {
    /// Aspect ratio of the dimension
    pub fn aspect(self) -> f64 {
        (self.width as f64) / (self.height as f64)
    }
}

/// Information for a canvas
pub struct Canvas {
    pub context: web_sys::CanvasRenderingContext2d,
    pub dim: Dimension,
}

impl Canvas {
    pub fn fill_rect(&self, start: (u32, u32), end: (u32, u32), color: [f32; 4]) {
        self.context.set_fill_style(
            &format!(
                "rgba({}, {}, {}, {})",
                color[0], color[1], color[2], color[3]
            )
            .into(),
        );
        self.context.fill_rect(
            start.0 as f64,
            start.1 as f64,
            (end.0 - start.0) as f64,
            (end.1 - start.1) as f64,
        );
    }
}
