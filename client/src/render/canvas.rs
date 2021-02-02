use traffloat::types::{Matrix, Vector};

/// The dimension of a canvas
#[derive(Debug, Clone, Copy)]
pub struct Dimension {
    pub width: u32,
    pub height: u32,
}

impl Dimension {
    /// Aspect ratio of the dimension
    pub fn aspect(self) -> f64 {
        (self.width as f64) / (self.height as f64)
    }

    pub fn as_vector(&self) -> Vector {
        Vector::new(self.width as f64, self.height as f64)
    }
}

/// Information for a canvas
pub struct Canvas {
    pub context: web_sys::CanvasRenderingContext2d,
    pub dim: Dimension,
}

impl Canvas {
    pub fn reset(&self, color: [f64; 4]) {
        self.context
            .reset_transform()
            .expect("Failed to reset canvas transformation");
        self.rect(
            (0., 0.),
            (self.dim.width as f64, self.dim.height as f64),
            color,
        );
    }

    fn color<T: From<String>>(rgba: [f64; 4]) -> T {
        format!(
            "rgba({}, {}, {}, {})",
            rgba[0] * 255.,
            rgba[1] * 255.,
            rgba[2] * 255.,
            rgba[3]
        )
        .into()
    }

    pub fn rect(&self, start: (f64, f64), end: (f64, f64), color: [f64; 4]) {
        self.context.set_fill_style(&Self::color(color));
        self.context
            .fill_rect(start.0, start.1, end.0 - start.0, end.1 - start.1);
    }

    /// Writes small print on the canvas
    pub fn note(&self, text: impl AsRef<str>, pos: (u32, u32), color: [f64; 4]) {
        self.context
            .reset_transform()
            .expect("Could not reset transformation matrix");
        self.context
            .set_stroke_style(&Self::color([0., 0., 0., 1.]));
        self.context.set_fill_style(&Self::color(color));
        self.context.set_font("12px sans-serif");
        self.context
            .stroke_text(text.as_ref(), pos.0 as f64, pos.1 as f64)
            .expect("Error writing text");
        self.context
            .fill_text(text.as_ref(), pos.0 as f64, pos.1 as f64)
            .expect("Error writing text");
    }

    fn set_transform(&self, matrix: Matrix) {
        self.context
            .set_transform(
                *matrix.index((0, 0)),
                *matrix.index((1, 0)),
                *matrix.index((0, 1)),
                *matrix.index((1, 1)),
                *matrix.index((0, 2)),
                *matrix.index((1, 2)),
            )
            .expect("Invalid transformation matrix used");
    }

    /// Draws an image with the given transformation matrix.
    ///
    /// The transformation matrix shall transform the unit square `[0, 1]^2`
    /// into the quadrilateral containing the image on the canvas.
    ///
    /// The image is first flippedg vertically within the unit square,
    /// then transformed with the given transformation matrix,
    /// then flipped vertically within the canvas.
    pub fn draw_image(&self, image: &impl Image, mut transform: Matrix) {
        transform.prepend_translation_mut(&Vector::new(0., 1.));
        transform.prepend_nonuniform_scaling_mut(&Vector::new(1., -1.));
        transform.append_translation_mut(&Vector::new(0., self.dim.height as f64 / -2.));
        transform.append_nonuniform_scaling_mut(&Vector::new(1., -1.));
        transform.append_translation_mut(&Vector::new(0., self.dim.height as f64 / 2.));

        self.set_transform(transform);

        if let Some(bitmap) = image.as_bitmap() {
            self.context
                .draw_image_with_image_bitmap_and_dw_and_dh(bitmap, 0., 0., 1., 1.)
                .expect("Could not draw bitmap");
        }
    }
}

pub trait Image {
    fn as_bitmap(&self) -> Option<&web_sys::ImageBitmap>;
}
