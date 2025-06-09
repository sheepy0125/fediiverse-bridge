use bardecoder::Decoder;
use cabbage::{F_TOP_HEIGHT, F_TOP_WIDTH};
use image::{GrayImage, RgbaImage};

pub struct Scanner {
    pub decoder: Decoder<RgbaImage, GrayImage, String>,
    pub image: RgbaImage,
}
impl Default for Scanner {
    fn default() -> Self {
        let image = RgbaImage::new(F_TOP_WIDTH as _, F_TOP_HEIGHT as _);
        let decoder = bardecoder::default_decoder();
        Self { decoder, image }
    }
}
impl Scanner {
    pub fn scan(&mut self) -> anyhow::Result<Option<String>> {
        let mut decoded = self.decoder.decode(&self.image);
        if let Some(Ok(frame)) = decoded.pop() {
            return Ok(Some(frame));
        }
        Ok(None)
    }
}
