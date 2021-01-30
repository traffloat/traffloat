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
    fn color<T: From<String>>(rgba: [f32; 4]) -> T {
        format!(
            "rgba({}, {}, {}, {})",
            rgba[0] * 255.,
            rgba[1] * 255.,
            rgba[2] * 255.,
            rgba[3]
        )
        .into()
    }

    pub fn rect(&self, start: (u32, u32), end: (u32, u32), color: [f32; 4]) {
        self.context.set_fill_style(&Self::color(color));
        self.context.fill_rect(
            start.0 as f64,
            start.1 as f64,
            (end.0 - start.0) as f64,
            (end.1 - start.1) as f64,
        );
    }

    pub fn note(&self, text: impl AsRef<str>, pos: (u32, u32), color: [f32; 4]) {
        self.context.set_fill_style(&Self::color(color));
        self.context.set_font("12px sans-serif");
        self.context
            .fill_text(text.as_ref(), pos.0 as f64, pos.1 as f64)
            .expect("Error writing text");
    }
}
