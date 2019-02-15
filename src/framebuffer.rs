use png::HasParameters;

use crate::RenderError;

pub struct Framebuffer {
    width: usize,
    height: usize,
    buf: Vec<[f32; 3]>,
}

fn f32_to_u8(val: f32) -> u8 {
    let val = f32::min(1.0, f32::max(0.0, val));
    (255.0 * val) as u8
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buf: vec![[0.0; 3]; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn buf(&self) -> &[[f32; 3]] {
        &self.buf
    }

    pub fn buf_mut(&mut self) -> &mut [[f32; 3]] {
        &mut self.buf
    }

    pub fn write_png<W: std::io::Write>(&self, w: W) -> Result<(), RenderError> {
        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(RenderError::Encode)?;

        let conv: Vec<_> = self
            .buf
            .iter()
            .map(|rgb| rgb.iter().map(|&v| f32_to_u8(v)))
            .flatten()
            .collect();
        writer.write_image_data(&conv).map_err(RenderError::Encode)?;
        Ok(())
    }
}
