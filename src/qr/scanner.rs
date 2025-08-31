use cabbage::{F_TOP_HEIGHT, F_TOP_WIDTH};
use image::{GrayImage, ImageBuffer, RgbaImage, buffer::ConvertBuffer};

pub struct Scanner {
    pub image: RgbaImage,
}
impl Default for Scanner {
    fn default() -> Self {
        let image = ImageBuffer::new(F_TOP_WIDTH as _, F_TOP_HEIGHT as _);
        Self { image }
    }
}
impl Scanner {
    pub fn scan(&mut self) -> Option<String> {
        let luma = self.image.convert();
        let mut prepared = rqrr::PreparedImage::prepare(luma);
        if let Some(grid) = prepared.detect_grids().pop() {
            let Ok((_metadata, decoded)) = grid.decode() else {
                return None;
            };
            return Some(decoded);
        }

        None
    }
}
